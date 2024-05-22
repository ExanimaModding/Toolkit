use std::{collections::HashMap, sync::RwLock};

use anyhow::{Error, Result};
use emf_types::plugin::PluginMessageRaw;
use once_cell::sync::Lazy;

use super::plugin_state::PluginState;

static PLUGIN_MANAGER: Lazy<RwLock<HashMap<String, PluginState>>> = Lazy::new(|| {
	let map = HashMap::new();

	RwLock::new(map)
});

pub struct PluginManager;

// TODO: Finish this properly.

impl PluginManager {
	pub unsafe fn register_plugin(&mut self, plugin_state: PluginState) -> Option<String> {
		let mut map = PLUGIN_MANAGER.write().unwrap();

		plugin_state.init?();

		let plugin_id = plugin_state.plugin_id.clone();

		map.insert(plugin_id.clone(), plugin_state);

		Some(plugin_id.clone())
	}

	pub fn get_state(&self, plugin_id: &str) -> Result<bool> {
		let map = PLUGIN_MANAGER.read().unwrap();

		if let Some(state) = map.get(plugin_id) {
			Ok(state.enabled)
		} else {
			Err(Error::msg("Plugin not found"))
		}
	}

	pub fn set_state(&self, plugin_id: &str, state: PluginState) {
		let mut map = PLUGIN_MANAGER.write().unwrap();

		map.insert(plugin_id.to_string(), state);
	}

	pub fn send_message(&self, message: *const PluginMessageRaw) {
		let message = unsafe { (*message).serialize() };
		dbg!(&message);
		let to = message.to.to_str().unwrap().to_owned();

		let map = PLUGIN_MANAGER.read().unwrap();

		if let Some(state) = map.get(&to) {
			let message = message.deserialize();
			state.send_message.unwrap()(&raw const message);
		}
	}
}
