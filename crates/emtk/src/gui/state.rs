use crate::config::AppSettings;

// TODO: refactor AppSettings
#[derive(Debug, Clone)]
pub struct AppState {
	pub settings: AppSettings,
}

impl Default for AppState {
	fn default() -> Self {
		Self {
			settings: AppSettings::read(),
		}
	}
}
