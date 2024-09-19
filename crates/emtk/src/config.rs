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
