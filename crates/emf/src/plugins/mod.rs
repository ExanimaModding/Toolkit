mod config;
pub mod manager;
mod parser;

use anyhow::*;
use log::*;
use path_clean::PathClean;
use std::{path::PathBuf, result::Result::Ok};

use crate::internal::utils::get_game_dir;

pub fn load_plugin(info: config::PluginInfo) -> Result<()> {
	info!(
		"Loading Plugin: {} ({})",
		info.config.plugin.name, info.config.plugin.id
	);

	if info.config.plugin.executable.is_none() {
		info!("Plugin does not have an executable. Skipping.");
		return Ok(());
	}

	unsafe {
		let executable = info.config.plugin.executable.as_ref().unwrap();
		let dll_path = PathBuf::from(&info.path).join(executable);
		let dll_path = dll_path.clean();

		// Security: Make sure the dll path is in the current mod folder.
		if !dll_path.starts_with(&info.path) {
			return Err(anyhow!("Invalid plugin path. {}", dll_path.display()));
		}

		info!("Loading DLL: {}", dll_path.display());
		let lib = libloading::Library::new(dll_path)?;

		let state = manager::PluginManager::add(manager::PluginState::new(lib, info)?);

		if let Some(state) = state {
			let writer: std::sync::MutexGuard<manager::PluginState> = state.lock().unwrap();
			(writer.enable)();

			Ok(())
		} else {
			Err(anyhow!("Failed to load plugin."))
		}
	}
}

pub fn read_plugin_configs() -> Result<Vec<config::PluginInfo>> {
	let mut configs = Vec::new();

	let path = get_game_dir().join("mods");
	if !path.exists() {
		std::fs::create_dir(&path).expect("error trying to create mods folder");
	}

	for entry in std::fs::read_dir(path)? {
		let entry = entry?;
		if entry.file_type()?.is_dir() {
			let config_path = entry.path().join("config.toml");
			if !config_path.exists() {
				continue;
			}

			let config = std::fs::read_to_string(&config_path)?;
			let config = parser::parse_plugin_config(&config);

			match config {
				Ok(config) => {
					configs.push(config::PluginInfo {
						config,
						path: entry.path().to_str().unwrap().to_owned(),
					});
				}
				Err(e) => {
					error!(
						"Failed to parse plugin config for {}. Error: {}",
						config_path.to_str().unwrap(),
						e
					);
					continue;
				}
			}
		}
	}

	Ok(configs)
}