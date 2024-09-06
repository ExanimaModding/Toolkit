pub mod changelog;
pub mod home;
pub mod progress;
pub mod settings;

use changelog::Changelog;
use home::Home;
use progress::Progress;
use settings::Settings;

#[derive(Debug, Clone)]
pub enum ScreenKind {
	Changelog,
	Home,
	Progress,
	Settings,
}

#[derive(Debug, Clone, strum::Display)]
pub enum Screen {
	Changelog(Changelog),
	Home(Home),
	Progress(Progress),
	Settings(Settings),
}

impl Default for Screen {
	fn default() -> Self {
		Self::Home(Home::default())
	}
}
