use std::{
	ffi::{self, CStr},
	fs, io,
	path::PathBuf,
	ptr,
	sync::Once,
};

use emcore::{
	Error,
	plugin::{self, Manifest},
};
use mlua::{Lua, Table};
use tracing::{error, info, instrument, trace, warn};

use crate::{internal::runtime, subscribe};

static TRACING_SUBSCRIBER: Once = Once::new();

#[instrument(level = "trace")]
#[unsafe(no_mangle)]
pub extern "C" fn EmfGetPluginList(mods_dir: *mut ffi::c_char) -> *mut ffi::c_char {
	TRACING_SUBSCRIBER.call_once(|| subscribe());

	if mods_dir.is_null() {
		error!("mods directory is null");
		return ptr::null_mut();
	}

	let Ok(mods_dir) = unsafe { CStr::from_ptr(mods_dir) }
		.to_str()
		.map_err(Error::msg("failed to get string from mods directory"))
		.map_err(|e| error!("{}", e))
	else {
		return ptr::null_mut();
	};

	let mods_dir = PathBuf::from(mods_dir);

	let Ok(manifests) = locate_plugins(mods_dir)
		.map_err(Error::msg("failed to locate plugins"))
		.map_err(|e| error!("{}", e))
	else {
		return ptr::null_mut();
	};

	let Ok(buffer) = ron::to_string(&manifests)
		.map_err(Error::msg(
			"failed to serialize plugin manifests into buffer",
		))
		.map_err(|e| error!("{}", e))
	else {
		return ptr::null_mut();
	};

	let Ok(buffer) = ffi::CString::new(buffer)
		.map_err(Error::msg("failed to create new C string for buffer"))
		.map_err(|e| error!("{}", e))
	else {
		return ptr::null_mut();
	};

	buffer.into_raw()
}

#[instrument(level = "trace")]
pub fn locate_plugins(mods_dir: PathBuf) -> Result<Vec<(plugin::Id, Manifest)>, io::Error> {
	trace!("{:#?}", mods_dir);
	if !mods_dir.is_dir() {
		return Err(io::Error::new(
			io::ErrorKind::NotFound,
			"no such path exists",
		));
	}

	let mut plugins = Vec::new();
	for entry in fs::read_dir(mods_dir)?.flatten() {
		trace!("{:#?}", entry);
		let entry_name = entry.file_name().display().to_string();
		let Ok(plugin_id) = plugin::Id::try_from(entry_name)
			.map_err(Error::msg("failed to get plugin id"))
			.map_err(|e| error!("{}", e))
		else {
			continue;
		};

		if !entry.path().is_dir() {
			warn!("{} is not a directory", plugin_id);
			continue;
		}

		let lua_file = entry.path().join(plugin::LUA);
		if !lua_file.is_file() {
			error!("failed to find {}'s {} file", plugin_id, plugin::LUA);
			continue;
		}

		// Run this in it's own lua runtime to prevent any conflicts with other plugins.
		let lua = Lua::new();

		let Ok(buffer) = fs::read_to_string(lua_file)
			.map_err(Error::msg(format!(
				"failed to read into buffer for lua file at {}",
				plugin_id
			)))
			.map_err(|e| error!("{}", e))
		else {
			continue;
		};
		let Ok(exports_table): Result<Table, _> = lua
			.load(&buffer)
			.eval()
			.map_err(anyhow::Error::new)
			.map_err(Error::msg(format!(
				"failed to evaluate lua expression from {}'s {} file",
				plugin_id,
				plugin::LUA
			)))
			.map_err(|e| error!("{}", e))
		else {
			continue;
		};

		let Ok(manifest_table) = exports_table
			.get::<Table>("manifest")
			.map_err(anyhow::Error::new)
			.map_err(Error::msg(format!(
				"failed to get manifest table from lua expression in {}'s {} file",
				plugin_id,
				plugin::LUA
			)))
			.map_err(|e| error!("{}", e))
		else {
			continue;
		};

		let Ok(manifest) =
			runtime::registry::parse_manifest(&plugin_id.to_string(), manifest_table)
				.map_err(|e| error!("{}", e))
		else {
			continue;
		};

		plugins.push((plugin_id, manifest));
	}
	trace!("{:#?}", &plugins);

	Ok(plugins)
}
