use emcore::plugin::{CManifest, GetPluginListResponse};
use std::{
	ffi::{self, CStr},
	path::PathBuf,
};

use crate::plugins::locate_plugins;

#[unsafe(no_mangle)]
pub extern "C" fn EMF_GetPluginList(mods_dir: *mut ffi::c_char) -> *mut GetPluginListResponse {
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

	dbg!(&plugins);

	let mut plugins = plugins
		.into_iter()
		.map(|plugin| CManifest::try_from(plugin))
		.flatten()
		.collect::<Vec<_>>();
	dbg!(&plugins);

	let plugins_len = plugins.len();

	let response = GetPluginListResponse {
		count: plugins_len,
		plugins: plugins.as_mut_ptr(),
	};

	Box::into_raw(Box::new(response))
}
