pub mod changelog;
pub mod explorer;
pub mod home;
pub mod progress;
pub mod settings;

use changelog::Changelog;
use explorer::Explorer;
use home::Home;
use progress::Progress;
use settings::Settings;

#[derive(Debug, Clone)]
pub enum ScreenKind {
	Changelog,
	Explorer,
	Home,
	Progress,
	Settings,
}

#[derive(strum::Display)]
pub enum Screen {
	Changelog(Changelog),
	Explorer(Explorer),
	Home(Home),
	Progress(Progress),
	Settings(Settings),
}

impl Default for Screen {
	fn default() -> Self {
		Self::Home(Home::default())
	}
}
