use mlua::Table;
use std::{io, path::PathBuf};

use emcore::plugin::{self, Manifest};

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
				let Ok(plugin_id) = plugin::Id::try_from(plugin_id) else {
					continue;
				};

				let path = entry.path();
				let Some(path) = path.to_str() else {
					continue;
				};

				let plugin_str = std::fs::read_to_string(config_path).unwrap();
				let lua = mlua::Lua::new();

				let Ok(Some(config)): Result<Option<Table>, _> = lua.load(&plugin_str).eval()
				else {
					continue;
				};

				let Ok(name): Result<String, _> = config.get::<String>("name") else {
					continue;
				};

				let Ok(version): Result<String, _> = config.get::<String>("version") else {
					continue;
				};

				let Ok(author): Result<String, _> = config.get::<String>("author") else {
					continue;
				};

				let dependencies = config
					.get::<Vec<String>>("dependencies")
					.unwrap_or_default()
					.into_iter()
					.map(plugin::Id::try_from)
					.flatten()
					.collect();

				let conflicts = config
					.get::<Vec<String>>("conflicts")
					.unwrap_or_default()
					.into_iter()
					.map(plugin::Id::try_from)
					.flatten()
					.collect();

				plugins.push(plugin::Manifest {
					id: plugin_id,
					name,
					version,
					author,
					dependencies,
					conflicts,
					path: path.to_string(),
				});
			}
		}
	}

	Ok(plugins)
}
