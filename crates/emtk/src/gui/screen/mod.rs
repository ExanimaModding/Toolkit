pub mod changelog;
pub mod home;
pub mod settings;

use changelog::Changelog;
use home::Home;
use settings::Settings;

#[derive(Debug, Clone)]
pub enum ScreenKind {
	Changelog,
	Home,
	Settings,
}

#[derive(Debug, Clone, strum::Display)]
pub enum Screen {
	Changelog(Changelog),
	Home(Home),
	Settings(Settings),
}

impl Default for Screen {
	fn default() -> Self {
		Self::Home(Home::default())
	}
}
