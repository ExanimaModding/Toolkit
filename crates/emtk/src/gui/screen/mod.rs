pub mod home;
pub mod settings;

use home::Home;
use settings::Settings;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ScreenKind {
	#[default]
	Home,
	Settings,
}

#[derive(Debug, Clone, strum::Display)]
pub enum Screen {
	Home(Home),
	Settings(Settings),
}

impl Default for Screen {
	fn default() -> Self {
		Self::Home(Home::default())
	}
}
