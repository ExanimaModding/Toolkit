use crate::gui::constants;
use iced::{
	widget::{
		self, button, horizontal_rule,
		markdown::{self},
		scrollable, text, Column,
	},
	Element, Task,
};

#[derive(Debug, Clone)]
pub enum Message {
	GetLatestRelease(GetLatestReleaseState),
	OpenUrl(String),
	ToggleChangelog,
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
pub struct Settings {
	pub latest_release: GetLatestReleaseState,
	expand_changelog: bool,
	changelog: Vec<markdown::Item>,
}

impl Settings {
	pub fn new() -> (Self, Task<Message>) {
		(
			Self::default(),
			Task::done(Message::GetLatestRelease(GetLatestReleaseState::NotStarted)),
		)
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
				self.changelog =
					markdown::parse(&format!("[View in browser]({})\n", release.html_url))
						.collect();

				let mut changelog: Vec<_> = markdown::parse(&release.body).collect();

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
			Message::ToggleChangelog => {
				self.expand_changelog = !self.expand_changelog;
				Task::none()
			}
			_ => Task::none(),
		};

		result.map(crate::gui::Message::Settings)
	}

	pub fn view(&self) -> Element<Message> {
		let col = Column::new()
			.push(text("Here you can configure the toolkit.").size(20))
			.push(horizontal_rule(1))
			.spacing(10);

		let col = col.push(self.version());

		// TODO: indicate changelog button is a collapsible button
		let col = col
			.push(button(text("Changelog")).on_press(Message::ToggleChangelog))
			.spacing(10);

		col.push_maybe(if self.expand_changelog {
			Some(self.changelog())
		} else {
			None
		})
		.into()
	}

	fn version(&self) -> Element<Message> {
		Column::new()
			.push(
				text(format!(
					"You're currently on version {}",
					constants::CARGO_PKG_VERSION
				))
				.size(20),
			)
			.push(self.get_latest_release(&self.latest_release))
			.push(horizontal_rule(1))
			.spacing(10)
			.into()
	}

	fn get_latest_release(&self, latest_release: &GetLatestReleaseState) -> Element<Message> {
		match &latest_release {
			GetLatestReleaseState::NotStarted => text("Checking for updates...").into(),
			GetLatestReleaseState::Loading => text("Checking for updates...").into(),
			GetLatestReleaseState::Loaded(release) => {
				let ver = semver::Version::parse(release.tag_name.trim_start_matches("v"))
					.unwrap_or(semver::Version::new(0, 0, 0));

				if ver <= semver::Version::parse(constants::CARGO_PKG_VERSION).unwrap() {
					let palette = iced::theme::Palette::CATPPUCCIN_MOCHA;
					return Column::new()
						.spacing(10.)
						.push(text("You're already up to date!"))
						.push(horizontal_rule(1.))
						// TODO: make changelog a modal
						.push(scrollable(
							widget::markdown(
								&self.changelog,
								widget::markdown::Settings::default(),
								markdown::Style {
									inline_code_highlight: markdown::Highlight {
										background: iced::Background::Color(palette.background),
										border: iced::Border::default(),
									},
									inline_code_padding: iced::Padding::default(),
									inline_code_color: palette.text,
									link_color: palette.primary,
								},
							)
							.map(|url| Message::OpenUrl(url.to_string())),
						))
						.into();
				}

				Column::new()
					.spacing(10.)
					.push(text(format!(
						"There's a new version available: {} (Published: {})",
						release.tag_name,
						release.published_at.format("%Y-%m-%d %H:%M:%S")
					)))
					.push(
						button(text("Download"))
							.on_press(Message::OpenUrl(release.html_url.clone()))
							.width(100.),
					)
			}
			.into(),
			GetLatestReleaseState::Error(error) => text(format!("Error: {}", error)).into(),
		}
	}

	fn changelog(&self) -> Element<Message> {
		match &self.latest_release {
			GetLatestReleaseState::NotStarted => text("Checking for updates...").into(),
			GetLatestReleaseState::Loading => text("Checking for updates...").into(),
			GetLatestReleaseState::Loaded(_) => {
				let palette = iced::theme::Palette::CATPPUCCIN_MOCHA;
				Column::new()
					.push(scrollable(
						widget::markdown(
							&self.changelog,
							widget::markdown::Settings::default(),
							markdown::Style {
								inline_code_highlight: markdown::Highlight {
									background: iced::Background::Color(palette.background),
									border: iced::Border::default(),
								},
								inline_code_padding: iced::Padding::default(),
								inline_code_color: palette.text,
								link_color: palette.primary,
							},
						)
						.map(|url| Message::OpenUrl(url.to_string())),
					))
					.spacing(10.)
			}
			.into(),
			GetLatestReleaseState::Error(error) => text(format!("Error: {}", error)).into(),
		}
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
