// TODO: Implement all PluginState properties and remove this.
#![allow(dead_code)]

use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};

use anyhow::Result;

use libloading::os::windows::Symbol;
use once_cell::sync::Lazy;
use safer_ffi::prelude::*;

use emf_types::config;

use super::write_plugin_config;

unsafe impl Send for PluginState {}

pub struct PluginState {
	pub loaded: bool,
	pub enabled: bool,
	pub info: config::PluginInfo,

	pub lib: libloading::Library,

	pub enable: Symbol<extern "C" fn() -> bool>,
	pub disable: Symbol<extern "C" fn() -> bool>,

	pub send_message: Option<Symbol<extern "C" fn(message: char_p::Box)>>,

	pub read_setting_bool: Option<Symbol<extern "C" fn(setting: char_p::Box) -> bool>>,
	pub read_setting_int: Option<Symbol<extern "C" fn(setting: char_p::Box) -> i64>>,
	pub read_setting_float: Option<Symbol<extern "C" fn(setting: char_p::Box) -> f64>>,
	pub read_setting_string: Option<Symbol<extern "C" fn(setting: char_p::Box) -> char_p::Box>>,

	pub write_setting_bool: Option<Symbol<extern "C" fn(setting: char_p::Box, value: bool)>>,
	pub write_setting_int: Option<Symbol<extern "C" fn(setting: char_p::Box, value: i64)>>,
	pub write_setting_float: Option<Symbol<extern "C" fn(setting: char_p::Box, value: f64)>>,
	pub write_setting_string:
		Option<Symbol<extern "C" fn(setting: char_p::Box, value: char_p::Box)>>,
}

impl PluginState {
	pub unsafe fn new(lib: libloading::Library, info: config::PluginInfo) -> Result<PluginState> {
		/// Helper macro to get a symbol from a library & turn it into a raw (no lifetime checks) reference.
		///
		/// Make sure that `lib` stays in scope as long as the returned reference is used or the function call will crash.
		macro_rules! sym {
			($name:expr, $type:ty) => {
				if let Ok(sym) = lib.get::<$type>($name) {
					Some(sym.into_raw())
				} else {
					None
				}
			};
		}

		let enable = sym!(b"enable", extern "C" fn() -> bool);
		let disable = sym!(b"disable", extern "C" fn() -> bool);

		let send_message = sym!(b"send_message", extern "C" fn(char_p::Box));

		let read_setting_bool = sym!(b"read_setting_bool", extern "C" fn(char_p::Box) -> bool);
		let read_setting_int = sym!(b"read_setting_int", extern "C" fn(char_p::Box) -> i64);
		let read_setting_float = sym!(b"read_setting_float", extern "C" fn(char_p::Box) -> f64);
		let read_setting_string = sym!(
			b"read_setting_string",
			extern "C" fn(char_p::Box) -> char_p::Box
		);

		let write_setting_bool = sym!(b"write_setting_bool", extern "C" fn(char_p::Box, bool));
		let write_setting_int = sym!(b"write_setting_int", extern "C" fn(char_p::Box, i64));
		let write_setting_float = sym!(b"write_setting_float", extern "C" fn(char_p::Box, f64));
		let write_setting_string = sym!(
			b"write_setting_string",
			extern "C" fn(char_p::Box, char_p::Box)
		);

		Ok(PluginState {
			loaded: false,
			enabled: false,
			info,
			lib,
			enable: enable.expect("Plugin does not have an enable function"),
			disable: disable.expect("Plugin does not have a disable function"),

			send_message,

			read_setting_bool,
			read_setting_int,
			read_setting_float,
			read_setting_string,

			write_setting_bool,
			write_setting_int,
			write_setting_float,
			write_setting_string,
		})
	}
}

static PLUGIN_MANAGER: Lazy<RwLock<HashMap<String, Arc<RwLock<PluginState>>>>> =
	Lazy::new(|| RwLock::new(HashMap::new()));

pub struct PluginManager;

impl PluginManager {
	pub fn get_ids() -> Vec<String> {
		let lock = PLUGIN_MANAGER.read().unwrap();
		lock.keys().cloned().collect()
	}

	pub fn get_info_for(id: &str) -> Option<config::PluginInfo> {
		let lock = PLUGIN_MANAGER.read().unwrap();
		let state = lock.get(id)?;

		let info = state.read().unwrap().info.clone();

		Some(info)
	}

	pub fn set_info_for(id: &str, info: config::PluginInfo) -> Option<()> {
		let lock = PLUGIN_MANAGER.read().unwrap();
		let state = lock.get(id)?;

		let original_info = state.read().unwrap().info.clone();

		state.write().unwrap().info = info.clone();

		match write_plugin_config(&info) {
			Ok(_) => {
				if info.config.plugin.enabled != original_info.config.plugin.enabled {
					PluginManager::send_message(
						id,
						if info.config.plugin.enabled {
							PluginMessage::Enable
						} else {
							PluginMessage::Disable
						},
					);
				}
				for (i, setting) in info.config.settings.iter().enumerate() {
					if setting.value != original_info.config.settings[i].value {
						PluginManager::send_message(
							id,
							PluginMessage::SettingChanged((
								setting.name.clone(),
								setting.value.clone().unwrap(),
							)),
						);
					}
				}

				Some(())
			}
			Err(e) => {
				log::error!("Failed to write plugin config: {}", e);
				None
			}
		}?;

		Some(())
	}

	pub fn add(plugin: PluginState) -> Option<Arc<RwLock<PluginState>>> {
		let id: String = plugin.info.config.plugin.id.to_owned();
		let mut writer = PLUGIN_MANAGER.write().unwrap();
		writer.insert(
			plugin.info.config.plugin.id.to_string(),
			Arc::new(RwLock::new(plugin)),
		);

		let plugin = writer.get(&id);

		plugin.map(Arc::clone)
	}

	pub fn get(id: &str) -> Option<Arc<RwLock<PluginState>>> {
		let lock = PLUGIN_MANAGER.read().unwrap();
		let state = lock.get(id)?;

		Some(Arc::clone(state))
	}

	pub fn send_message(id: &str, message: PluginMessage) {
		let lock = PLUGIN_MANAGER.read().unwrap();
		let state = lock.get(id).unwrap();

		let state = state.read().unwrap();

		macro_rules! fn_if_exists {
			($fn:expr $(, $args:expr)*)  => {
				if let Some(fn_ptr) = $fn {
					fn_ptr($($args),*);
				}
			};
		}

		match message {
			PluginMessage::Message(message) => {
				fn_if_exists!(&state.send_message, char_p::new(message))
			}
			PluginMessage::Enable => {
				(state.enable)();
			}
			PluginMessage::Disable => {
				(state.disable)();
			}
			PluginMessage::SettingChanged((key, value)) => match value {
				config::PluginConfigSettingValue::Boolean(value) => {
					fn_if_exists!(&state.write_setting_bool, char_p::new(key), value);
				}
				config::PluginConfigSettingValue::Float(value) => {
					fn_if_exists!(&state.write_setting_float, char_p::new(key), value);
				}
				config::PluginConfigSettingValue::Integer(value) => {
					fn_if_exists!(&state.write_setting_int, char_p::new(key), value);
				}
				config::PluginConfigSettingValue::String(value) => {
					fn_if_exists!(
						&state.write_setting_string,
						char_p::new(key),
						char_p::new(value)
					);
				}
			},
		};
	}
}

pub enum PluginMessage {
	Message(String),
	Enable,
	Disable,
	SettingChanged((String, config::PluginConfigSettingValue)),
}
