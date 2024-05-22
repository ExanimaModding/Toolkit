use std::borrow::Borrow;

use anyhow::{Error, Result};
use emf_types::plugin::{GetPluginInfoFn, OnMessageFn, PluginInitFn};

use std::ffi::CStr;

use log::*;

use self::plugin_state::PluginState;

pub mod plugin_manager;
pub mod plugin_state;

// use emf_types::{load_root_module_in_directory, RModEntrypoint};

// pub fn init_mods() {
// 	load_module(r"w:\Exanima\mods\target\debug\");
// }

// pub fn load_module(path: &str) {
// 	let library: RModEntrypoint = load_root_module_in_directory(path.as_ref())
// 		.unwrap_or_else(|e| panic!("Failed to load module: {}", e));

// 	let library = library.new();
// }

// TODO: Implement properly

pub unsafe fn init_dll_plugins() -> Result<u32, Error> {
	let lib = libloading::Library::new(r"W:\Exanima\DLLPlugins\target\debug\godhand.dll")?;

	let mut plugin = PluginState {
		enabled: false,
		loaded: true,
		plugin_id: "com.megu.dll_plugins_test".to_owned(),
		lib: None,

		init: None,
		get_info: None,
		send_message: None,
	};

	let get_plugin_info = *lib.get::<GetPluginInfoFn>(b"get_plugin_info")?;
	let init = *lib.get::<PluginInitFn>(b"on_init")?;
	let send_message = *lib.get::<OnMessageFn>(b"on_message")?;

	plugin.init = Some(init);
	plugin.get_info = Some(get_plugin_info);
	plugin.send_message = Some(send_message);

	plugin.lib = Some(lib);

	let plugin_info = get_plugin_info();
	info!("Plugin info: {:#?}", *plugin_info);

	plugin_manager::PluginManager.register_plugin(plugin);

	Ok(1337)
}
