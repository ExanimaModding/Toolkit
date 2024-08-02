use iced::{
	theme,
	widget::{
		self,
		markdown::{self},
		scrollable, Column, Text,
	},
	Element, Task,
};

#[derive(Debug, Clone)]
pub enum Message {
	GetLatestRelease(GetLatestReleaseState),
	OpenUrl(String),
}

impl Default for Message {
	fn default() -> Self {
		Message::GetLatestRelease(GetLatestReleaseState::NotStarted)
	}
}

#[derive(Debug, Default, Clone)]
pub enum GetLatestReleaseState {
	#[default]
	NotStarted,
	Loading,
	Loaded(Release),
	Error(String),
}

#[derive(Debug, Default, Clone)]
pub struct Changelog {
	pub latest_release: GetLatestReleaseState,
	changelog: Vec<markdown::Item>,
}

impl Changelog {
	pub fn new() -> (Self, Task<Message>) {
		(
			Self::default(),
			Task::done(Message::GetLatestRelease(GetLatestReleaseState::NotStarted)),
		)
	}

	pub fn view(&self) -> Element<Message> {
		match &self.latest_release {
			GetLatestReleaseState::NotStarted => Text::new("Checking for updates...").into(),
			GetLatestReleaseState::Loading => Text::new("Checking for updates...").into(),
			GetLatestReleaseState::Loaded(_) => {
				Column::new()
					.push(scrollable(
						widget::markdown(&self.changelog, widget::markdown::Settings::default())
							.map(|url| Message::OpenUrl(url.to_string())),
					))
					.spacing(10.)
			}
			.into(),
			GetLatestReleaseState::Error(error) => Text::new(format!("Error: {}", error)).into(),
		}
	}

	pub fn update(
		&mut self,
		_app_state: &mut crate::gui::state::AppState,
		message: Message,
	) -> Task<crate::gui::Message> {
		let result = match message {
			Message::GetLatestRelease(GetLatestReleaseState::NotStarted) => {
				log::info!("Checking for updates...");
				self.latest_release = GetLatestReleaseState::Loading;
				Task::future(get_latest_release()).map(|result| match result {
					Ok(release) => {
						log::info!("Latest release: {}", release.tag_name);
						Message::GetLatestRelease(GetLatestReleaseState::Loaded(release))
					}
					Err(error) => {
						log::error!("Error checking for updates: {}", error);
						Message::GetLatestRelease(GetLatestReleaseState::Error(error.to_string()))
					}
				})
			}
			Message::GetLatestRelease(GetLatestReleaseState::Loaded(release)) => {
				self.changelog = markdown::parse(
					&format!("[View in browser]({})\n", release.html_url),
					theme::Palette::CATPPUCCIN_MOCHA,
				)
				.collect();

				let mut changelog: Vec<_> =
					markdown::parse(&release.body, theme::Palette::CATPPUCCIN_MOCHA).collect();

				self.changelog.append(&mut changelog);
				log::info!("Latest release: {}", release.tag_name);
				self.latest_release = GetLatestReleaseState::Loaded(release);
				Task::none()
			}
			Message::GetLatestRelease(GetLatestReleaseState::Error(error)) => {
				log::error!("Error checking for updates: {}", error);
				self.latest_release = GetLatestReleaseState::Error(error);
				Task::none()
			}
			Message::OpenUrl(url) => {
				log::info!("Opening URL: {}", url);
				open::that(url).unwrap();
				Task::none()
			}
			_ => Task::none(),
		};

		result.map(crate::gui::Message::Changelog)
	}
}

#[derive(Debug, Default, Clone, ureq::serde::Deserialize)]
pub struct Release {
	pub tag_name: String,
	pub html_url: String,
	pub body: String,
	pub published_at: chrono::DateTime<chrono::Utc>,
}

async fn get_latest_release() -> anyhow::Result<Release> {
	let url = "https://codeberg.org/api/v1/repos/ExanimaModding/Toolkit/releases/latest";

	let release: Release = ureq::get(url).call()?.into_json()?;

	Ok(release)
}
