use std::{fs, path};

use emf_types::config::PluginInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
	pub exanima_exe: Option<String>,
	pub launcher: Option<Launcher>,
	pub load_order: Vec<(String, bool)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Launcher {
	pub developer: bool,
	pub explain: bool,
	pub theme: String,
}

impl Default for Launcher {
	fn default() -> Self {
		Self {
			developer: false,
			explain: false,
			theme: "light".to_string(),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppSettings {
	pub exanima_exe: Option<String>,
	pub mod_load_order: Vec<String>,
	#[serde(skip, default)]
	pub mods: Vec<ModSetting>,
}

#[derive(Debug, Clone)]
pub struct ModSetting {
	pub info: emf_types::config::PluginInfo,
}

impl AppSettings {
	pub fn read() -> Self {
		let settings_dir = get_settings_dir();
		let settings_path = settings_dir.join("launcher.toml");

		let mut settings = if settings_path.exists() {
			let settings_str =
				fs::read_to_string(&settings_path).expect("Could not read settings file");
			toml::from_str(&settings_str).expect("Could not parse settings file")
		} else {
			let settings = Self::default();
			settings.save();
			settings
		};

		if let Some(exanima_exe) = &settings.exanima_exe {
			settings.mods = load_mod_info(exanima_exe);
		}

		settings
	}

	pub fn save(&self) {
		let settings_dir = get_settings_dir();
		let settings_path = settings_dir.join("launcher.toml");

		let settings_str = toml::to_string(self).expect("Could not serialize settings");
		fs::write(&settings_path, settings_str).expect("Could not write settings file");
	}
}

fn get_settings_dir() -> path::PathBuf {
	let mut settings_dir = dirs::config_dir().expect("Could not find config directory");
	settings_dir.push("Exanima Modding Toolkit");

	if !settings_dir.exists() {
		fs::create_dir_all(&settings_dir).expect("Could not create settings directory");
	}

	settings_dir
}

fn load_mod_info(exanima_exe: &str) -> Vec<ModSetting> {
	let mods_dir = path::Path::new(exanima_exe).parent().unwrap().join("mods");

	let mut mods = Vec::new();

	for entry in fs::read_dir(mods_dir).expect("Could not read mods directory") {
		let entry = entry.expect("Could not read entry");
		let path = entry.path();

		if path.is_dir() {
			let mod_info_path = path.join("config.toml");

			if mod_info_path.exists() {
				let mod_info_str =
					fs::read_to_string(&mod_info_path).expect("Could not read mod config.toml");
				let mod_info: emf_types::config::PluginConfig =
					toml::from_str(&mod_info_str).expect("Could not parse mod config.toml");

				let mod_setting = ModSetting {
					info: PluginInfo {
						config: mod_info,
						path: path.to_string_lossy().to_string(),
					},
				};
				mods.push(mod_setting);
			}
		}
	}

	mods
}
