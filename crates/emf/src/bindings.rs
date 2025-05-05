use std::{
	ffi::{self, CStr},
	io,
	path::PathBuf,
};

use emcore::plugin::Manifest;
use mlua::Table;

use crate::internal::runtime;

#[unsafe(no_mangle)]
pub extern "C" fn EMF_GetPluginList(mods_dir: *mut ffi::c_char) -> *mut ffi::c_char {
	if mods_dir.is_null() {
		return std::ptr::null_mut();
	}

	let Ok(mods_dir) = unsafe { CStr::from_ptr(mods_dir) }.to_str() else {
		return std::ptr::null_mut();
	};

	let mods_dir = PathBuf::from(mods_dir);

	let Ok(plugins) = locate_plugins(mods_dir) else {
		return std::ptr::null_mut();
	};

	let Ok(response) = ron::to_string(&plugins) else {
		return std::ptr::null_mut();
	};

	let Ok(response) = ffi::CString::new(response) else {
		return std::ptr::null_mut();
	};

	response.into_raw()
}

pub fn locate_plugins(mods_dir: PathBuf) -> Result<Vec<Manifest>, io::Error> {
	if !mods_dir.exists() {
		return Err(io::Error::new(
			io::ErrorKind::NotFound,
			"mods directory does not exist",
		));
	}

	let mut plugins = Vec::new();

	for entry in std::fs::read_dir(mods_dir).unwrap() {
		let entry = entry.unwrap();
		if entry.file_type().unwrap().is_dir() {
			let config_path = entry.path().join("plugin.lua");
			if config_path.exists() {
				let plugin_id = entry.file_name();
				let Some(plugin_id) = plugin_id.to_str() else {
					continue;
				};

				let Ok(plugin_str) = std::fs::read_to_string(config_path) else {
					continue;
				};

				// Run this in it's own lua runtime to prevent any conflicts with other plugins.
				let lua = mlua::Lua::new();

				let Ok(exports): Result<Table, _> = lua.load(&plugin_str).eval() else {
					continue;
				};

				let Ok(manifest) = exports.get::<Table>("manifest") else {
					continue;
				};

				let Ok(manifest) = runtime::registry::parse_manifest(plugin_id, manifest) else {
					continue;
				};

				plugins.push(manifest);
			}
		}
	}

	Ok(plugins)
}
