use super::changelog::GetLatestReleaseState;
use crate::gui::constants;

use iced::{
	widget::{
		self,
		markdown::{self},
		scrollable, Button, Column, Container, Row, Rule, Text,
	},
	Element, Task,
};

#[derive(Debug, Clone)]
pub enum Message {
	OpenUrl(String),
	StartGame(GameStartType),
}

#[derive(Debug, Clone)]
pub enum GameStartType {
	Modded,
	Vanilla,
}

#[derive(Debug, Clone, Default)]
pub enum GameStartState {
	#[default]
	NotStarted,
	Loading,
	Loaded,
	Error(String),
}

#[derive(Debug, Default, Clone)]
pub struct Home {
	changelog: Vec<markdown::Item>,
	game_start_state: GameStartState,
}

impl Home {
	pub fn view(
		&self,
		latest_release: &super::changelog::GetLatestReleaseState,
	) -> Element<Message> {
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
			.push(self.get_latest_release(latest_release))
			.push(Rule::horizontal(1.))
			.push(self.show_home_section())
			.into()
	}

	fn show_home_section(&self) -> Element<Message> {
		Column::new()
			.push(
				Container::new(Text::new("").height(iced::Length::Fill)).height(iced::Length::Fill),
			)
			.push(self.play_buttons())
			.into()
	}

	fn play_buttons(&self) -> Element<Message> {
		match self.game_start_state {
			GameStartState::NotStarted => Row::new()
				.spacing(10.)
				.push(
					Button::new(Text::new("Play Modded").size(20))
						.on_press(Message::StartGame(GameStartType::Modded)),
				)
				.push(
					Button::new(Text::new("Play Unmodded").size(20))
						.on_press(Message::StartGame(GameStartType::Vanilla)),
				)
				.into(),
			_ => Row::new()
				.spacing(10.)
				.push(Button::new(Text::new("Play Modded").size(20)))
				.push(Button::new(Text::new("Play Unmodded").size(20)))
				.into(),
		}
	}

	fn get_latest_release(
		&self,
		latest_release: &super::changelog::GetLatestReleaseState,
	) -> Element<Message> {
		match &latest_release {
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

	pub fn update(
		&mut self,
		_app_state: &mut crate::gui::state::AppState,
		message: Message,
	) -> Task<crate::gui::Message> {
		let result = match message {
			Message::OpenUrl(url) => {
				log::info!("Opening URL: {}", url);
				open::that(url).unwrap();
				Task::none()
			}
			Message::StartGame(GameStartType::Modded) => {
				self.game_start_state = GameStartState::Loading;
				log::info!("Starting modded Exanima...");
				Task::none()
			}
			Message::StartGame(GameStartType::Vanilla) => {
				self.game_start_state = GameStartState::Loading;
				log::info!("Starting vanilla Exanima...");
				Task::none()
			}
		};

		result.map(crate::gui::Message::HomePage)
	}
}
