use std::{
	ffi::{self, CStr},
	fs, io,
	path::PathBuf,
	ptr,
};

use emcore::plugin::{self, Manifest};
use mlua::Table;
use tracing::{error, instrument, trace, warn};

use crate::internal::runtime;

#[instrument(level = "trace")]
#[unsafe(no_mangle)]
pub extern "C" fn EmfGetPluginList(mods_dir: *mut ffi::c_char) -> *mut ffi::c_char {
	if mods_dir.is_null() {
		return ptr::null_mut();
	}

	let Ok(mods_dir) = unsafe { CStr::from_ptr(mods_dir) }.to_str() else {
		return ptr::null_mut();
	};

	let mods_dir = PathBuf::from(mods_dir);

	let Ok(plugins) = locate_plugins(mods_dir) else {
		return ptr::null_mut();
	};

	let Ok(response) = ron::to_string(&plugins) else {
		return ptr::null_mut();
	};

	let Ok(response) = ffi::CString::new(response) else {
		return ptr::null_mut();
	};

	response.into_raw()
}

#[instrument(level = "trace")]
pub fn locate_plugins(mods_dir: PathBuf) -> Result<Vec<(plugin::Id, Manifest)>, io::Error> {
	trace!("{:#?}", &mods_dir);
	if !mods_dir.is_dir() {
		return Err(io::Error::new(
			io::ErrorKind::NotFound,
			"mods directory does not exist",
		));
	}

	let mut plugins = Vec::new();
	for entry in fs::read_dir(mods_dir)?.flatten() {
		trace!("{:#?}", &entry);
		if let Ok(file_type) = entry.file_type()
			&& file_type.is_dir()
		{
			let config_path = entry.path().join("plugin.lua");
			if config_path.exists() {
				let entry_name = entry.file_name().display().to_string();
				let Ok(plugin_id) = plugin::Id::try_from(entry_name).map_err(|e| warn!("{}", e))
				else {
					continue;
				};

				// Run this in it's own lua runtime to prevent any conflicts with other plugins.
				let lua = mlua::Lua::new();

				let Ok(buffer) = fs::read_to_string(config_path).map_err(|_| {
					error!("failed to read into buffer for lua file at {}", plugin_id)
				}) else {
					continue;
				};
				let Ok(exports): Result<Table, _> = lua.load(&buffer).eval().map_err(|_| {
					error!(
						"failed to evaluate buffer as lua source code at {}",
						plugin_id
					)
				}) else {
					continue;
				};

				let Ok(manifest) = exports.get::<Table>("manifest").map_err(|_| {
					error!(
						"failed to get manifest from lua source code at {}",
						plugin_id
					)
				}) else {
					continue;
				};

				let Ok(manifest) =
					runtime::registry::parse_manifest(&plugin_id.to_string(), manifest)
						.map_err(|e| error!("{}", e))
				else {
					continue;
				};

				plugins.push((plugin_id, manifest));
			}
		}
	}
	trace!("{:#?}", &plugins);

	Ok(plugins)
}
