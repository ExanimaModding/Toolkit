pub mod changelog;
pub mod confirm;
pub mod explorer;
pub mod mods;
pub mod progress;
pub mod settings;

use changelog::Changelog;
use explorer::Explorer;
use mods::Mods;
use progress::Progress;
use settings::Settings;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ScreenKind {
	Changelog,
	Explorer,
	Mods,
	Progress,
	Settings,
}

#[derive(strum::Display)]
pub enum Screen {
	Changelog(Changelog),
	Explorer(Explorer),
	Mods(Mods),
	Progress(Progress),
	Settings(Settings),
}

impl Default for Screen {
	fn default() -> Self {
		Self::Mods(Mods::default())
	}
}
