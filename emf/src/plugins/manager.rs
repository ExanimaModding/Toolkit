// TODO: Implement all PluginState properties and remove this.
#![allow(dead_code)]

use std::{
	collections::HashMap,
	sync::{Arc, Mutex, RwLock},
};

use anyhow::Result;

use libloading::os::windows::Symbol;
use once_cell::sync::Lazy;
use safer_ffi::prelude::repr_c;

use super::config;

unsafe impl Send for PluginState {}

pub struct PluginState {
	pub loaded: bool,
	pub enabled: bool,
	pub info: config::PluginInfo,

	pub lib: libloading::Library,

	pub enable: Symbol<extern "C" fn() -> bool>,
	pub disable: Symbol<extern "C" fn() -> bool>,

	pub send_message: Option<Symbol<extern "C" fn(message: repr_c::String)>>,

	pub read_setting_bool: Option<Symbol<extern "C" fn(setting: repr_c::String) -> bool>>,
	pub read_setting_int: Option<Symbol<extern "C" fn(setting: repr_c::String) -> i64>>,
	pub read_setting_float: Option<Symbol<extern "C" fn(setting: repr_c::String) -> f64>>,
	pub read_setting_string:
		Option<Symbol<extern "C" fn(setting: repr_c::String) -> repr_c::String>>,

	pub write_setting_bool: Option<Symbol<extern "C" fn(setting: repr_c::String, value: bool)>>,
	pub write_setting_int: Option<Symbol<extern "C" fn(setting: repr_c::String, value: i64)>>,
	pub write_setting_float: Option<Symbol<extern "C" fn(setting: repr_c::String, value: f64)>>,
	pub write_setting_string:
		Option<Symbol<extern "C" fn(setting: repr_c::String, value: repr_c::String)>>,
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

		let send_message = sym!(b"send_message", extern "C" fn(repr_c::String));

		let read_setting_bool = sym!(b"read_setting_bool", extern "C" fn(repr_c::String) -> bool);
		let read_setting_int = sym!(b"read_setting_int", extern "C" fn(repr_c::String) -> i64);
		let read_setting_float = sym!(b"read_setting_float", extern "C" fn(repr_c::String) -> f64);
		let read_setting_string = sym!(
			b"read_setting_string",
			extern "C" fn(repr_c::String) -> repr_c::String
		);

		let write_setting_bool = sym!(b"write_setting_bool", extern "C" fn(repr_c::String, bool));
		let write_setting_int = sym!(b"write_setting_int", extern "C" fn(repr_c::String, i64));
		let write_setting_float = sym!(b"write_setting_float", extern "C" fn(repr_c::String, f64));
		let write_setting_string = sym!(
			b"write_setting_string",
			extern "C" fn(repr_c::String, repr_c::String)
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

static PLUGIN_MANAGER: Lazy<RwLock<HashMap<String, Arc<Mutex<PluginState>>>>> =
	Lazy::new(|| RwLock::new(HashMap::new()));

pub struct PluginManager;

impl PluginManager {
	pub fn add(plugin: PluginState) -> Option<Arc<Mutex<PluginState>>> {
		let id: String = plugin.info.config.plugin.id.to_owned();
		let mut writer = PLUGIN_MANAGER.write().unwrap();
		writer.insert(
			plugin.info.config.plugin.id.to_string(),
			Arc::new(Mutex::new(plugin)),
		);

		let plugin = writer.get(&id);

		plugin.map(Arc::clone)
	}

	pub fn get(id: &str) -> Option<Arc<Mutex<PluginState>>> {
		let lock = PLUGIN_MANAGER.read().unwrap();
		let state = lock.get(id)?;

		Some(Arc::clone(state))
	}
}
