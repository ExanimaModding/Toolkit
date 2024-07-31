use iced::{
	theme,
	widget::{
		self, container,
		markdown::{self, Url},
		scrollable,
		text::Highlighter,
		Button, Column, Rule, Scrollable, Text,
	},
	Element, Renderer, Subscription, Task, Theme,
};

use crate::gui::constants;

#[derive(Debug, Clone)]
pub enum Message {
	GetLatestRelease(GetLatestReleaseState),
	OpenUrl(String),
	None,
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
pub struct Home {
	latest_release: GetLatestReleaseState,
	changelog: Vec<markdown::Item>,
}

impl Home {
	pub fn new() -> (Self, Task<Message>) {
		(
			Self::default(),
			Task::done(Message::GetLatestRelease(GetLatestReleaseState::NotStarted)),
		)
	}

	pub fn view(&self) -> Element<Message> {
		Column::new()
			.spacing(10.)
			.push(Text::new("Welcome to the Exanima Modding Toolkit Launcher!").size(20))
			.push(
				Text::new(format!(
					"You're currently on version {}",
					constants::CARGO_PKG_VERSION
				))
				.size(20),
			)
			.push(self.get_latest_release())
			.into()
	}

	fn get_latest_release(&self) -> Element<Message> {
		match &self.latest_release {
			GetLatestReleaseState::NotStarted => Text::new("Checking for updates...").into(),
			GetLatestReleaseState::Loading => Text::new("Checking for updates...").into(),
			GetLatestReleaseState::Loaded(release) => {
				let ver = semver::Version::parse(release.tag_name.trim_start_matches("v"))
					.unwrap_or(semver::Version::new(0, 0, 0));

				if ver <= semver::Version::parse(constants::CARGO_PKG_VERSION).unwrap() {
					return Column::new()
						.spacing(10.)
						.push(Text::new("You're already up to date!"))
						.push(Rule::horizontal(1.))
						.push(scrollable(
							widget::markdown(
								&self.changelog,
								widget::markdown::Settings::default(),
							)
							.map(|url| Message::OpenUrl(url.to_string())),
						))
						.into();
				}

				Column::new()
					.spacing(10.)
					.push(Text::new(format!(
						"There's a new version available: {} (Published: {})",
						release.tag_name,
						release.published_at.format("%Y-%m-%d %H:%M:%S")
					)))
					.push(
						Button::new(Text::new("Download"))
							.on_press(Message::OpenUrl(release.html_url.clone()))
							.width(100.),
					)
			}
			.into(),
			GetLatestReleaseState::Error(error) => Text::new(format!("Error: {}", error)).into(),
		}
	}

	pub fn update(&mut self, message: Message) -> Task<crate::gui::Message> {
		dbg!(&message);
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

		result.map(crate::gui::Message::HomePage)
	}
}

#[derive(Debug, Default, Clone, ureq::serde::Deserialize)]
pub struct Release {
	tag_name: String,
	html_url: String,
	body: String,
	published_at: chrono::DateTime<chrono::Utc>,
}

async fn get_latest_release() -> anyhow::Result<Release> {
	let url = "https://codeberg.org/api/v1/repos/ExanimaModding/Toolkit/releases/latest";

	let release: Release = ureq::get(url).call()?.into_json()?;

	Ok(release)
}
