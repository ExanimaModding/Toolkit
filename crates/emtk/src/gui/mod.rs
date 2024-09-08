mod constants;
mod screen;
mod state;
mod widget;

use iced::{
	event, theme,
	widget::{button, container, horizontal_rule, markdown, scrollable, text, Column, Row},
	window, Background, Border, Element, Length, Padding, Size, Subscription, Task, Theme,
};
use lilt::{Animated, Easing};
use screen::{
	changelog::{self, Changelog},
	home::{self, Home},
	progress::{self, Progress},
	settings::{self, Settings},
	Screen, ScreenKind,
};
use std::time::Instant;
use widget::modal::modal;

// TODO: animate scrolling in scrollbars
static ICON: &[u8] = include_bytes!("../../../../assets/images/corro.ico");
/// The animation duration for fade transitions in milliseconds.
pub static FADE_DURATION: u64 = 100;

pub(crate) async fn start_gui() -> iced::Result {
	let image = image::load_from_memory(ICON).unwrap();
	let icon =
		window::icon::from_rgba(image.as_bytes().to_vec(), image.height(), image.width()).unwrap();

	iced::application(Emtk::title, Emtk::update, Emtk::view)
		.theme(Emtk::theme)
		.window(window::Settings {
			icon: Some(icon),
			..Default::default()
		})
		.subscription(Emtk::subscription)
		.run_with(Emtk::new)
}

#[derive(Debug, Default, Clone)]
pub enum GetLatestReleaseState {
	#[default]
	NotStarted,
	Loading,
	Loaded(Release),
	Error(String),
}

#[derive(Debug, Default, Clone, ureq::serde::Deserialize)]
pub struct Release {
	pub tag_name: String,
	pub html_url: String,
	pub body: String,
	pub published_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct Emtk {
	app_state: state::AppState,
	changelog: Vec<markdown::Item>,
	fade: Animated<bool, Instant>,
	latest_release: GetLatestReleaseState,
	modal: Option<Screen>,
	screen: Screen,
	window_size: Size,
}

#[derive(Debug, Clone)]
pub enum Message {
	Changelog(changelog::Message),
	ExanimaLaunched,
	GetLatestRelease(GetLatestReleaseState),
	Home(home::Message),
	IcedEvent(iced::Event),
	LinkClicked(String),
	ModalChanged(ScreenKind),
	ModalCleanup,
	ModalClosed,
	Nothing,
	Progress(progress::Message),
	ScreenChanged(ScreenKind),
	Settings(settings::Message),
	StartGame(GameStartType),
	Tick,
}

impl Emtk {
	pub fn new() -> (Self, Task<Message>) {
		let emtk = Self::default();
		let settings = emtk.app_state.settings.clone();
		(
			emtk,
			// TODO: refactor
			// Task::batch([
			// 	Task::done(screen::settings::Message::default()).map(Message::Settings),
			// 	Task::done(screen::home::Message::LoadSettings(settings.clone()))
			// 		.map(Message::Home),
			// ]),
			Task::done(Message::GetLatestRelease(GetLatestReleaseState::NotStarted)),
		)
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		let now = Instant::now();

		match message {
			Message::Changelog(message) => match &mut self.modal {
				Some(screen) => match screen {
					Screen::Changelog(changelog) => {
						let (task, action) = changelog.update(message);
						let action = match action {
							changelog::Action::LinkClicked(url) => {
								Task::done(Message::LinkClicked(url))
							}
							changelog::Action::None => Task::none(),
						};
						Task::batch([task.map(Message::Changelog), action])
					}
					_ => Task::none(),
				},
				None => Task::none(),
			},
			Message::ExanimaLaunched => {
				// TODO: launch exanima
				// crate::launch_exanima();
				log::info!("Launching exanima...");
				Task::none()
			}
			Message::GetLatestRelease(state) => match state {
				GetLatestReleaseState::NotStarted => {
					log::info!("Checking for updates...");
					self.latest_release = GetLatestReleaseState::Loading;
					Task::future(get_latest_release()).map(|result| match result {
						Ok(release) => {
							log::info!("Latest release: {}", release.tag_name);
							Message::GetLatestRelease(GetLatestReleaseState::Loaded(release))
						}
						Err(error) => {
							log::error!("Error checking for updates: {}", error);
							Message::GetLatestRelease(GetLatestReleaseState::Error(
								error.to_string(),
							))
						}
					})
				}
				GetLatestReleaseState::Loading => Task::none(),
				GetLatestReleaseState::Loaded(release) => {
					self.changelog =
						markdown::parse(&format!("[View in browser]({})\n", release.html_url))
							.collect();

					let mut changelog: Vec<_> = markdown::parse(&release.body).collect();

					self.changelog.append(&mut changelog);
					log::info!("Latest release: {}", release.tag_name);
					self.latest_release = GetLatestReleaseState::Loaded(release);
					Task::none()
				}
				GetLatestReleaseState::Error(error) => {
					log::error!("Error checking for updates: {}", error);
					self.latest_release = GetLatestReleaseState::Error(error);
					Task::none()
				}
			},
			Message::Home(message) => match &mut self.screen {
				Screen::Home(home) => home.update(message, &mut self.app_state).map(Message::Home),
				_ => Task::none(),
			},
			Message::IcedEvent(event) => match event {
				iced::Event::Window(event) => match event {
					window::Event::Resized(size) => {
						self.window_size = size;
						let Some(screen) = &mut self.modal else {
							return Task::none();
						};
						match screen {
							Screen::Changelog(changelog) => {
								let width = size.width * 0.8;
								let height = size.height * 0.8;
								let size = Size::new(width, height);
								let (task, _action) =
									changelog.update(changelog::Message::SizeChanged(size));
								task.map(Message::Changelog)
							}
							Screen::Progress(progress) => {
								let width = size.width * 0.8;
								let size = Size::new(width, 0.);
								progress.update(progress::Message::SizeChanged(size));
								Task::none()
							}
							_ => Task::none(),
						}
					}
					_ => Task::none(),
				},
				_ => Task::none(),
			},
			Message::LinkClicked(url) => {
				log::info!("Opening URL: {}", url);
				open::that(url).unwrap();
				Task::none()
			}
			Message::ModalChanged(kind) => match kind {
				ScreenKind::Changelog => {
					self.fade.transition(true, now);
					self.modal = Some(Screen::Changelog(Changelog::new(
						self.changelog.clone(),
						self.latest_release.clone(),
						Some(self.window_size * 0.8),
					)));
					Task::none()
				}
				_ => Task::none(),
			},
			Message::ModalCleanup => {
				self.modal = None;
				Task::none()
			}
			Message::ModalClosed => {
				self.fade.transition(false, now);
				match &mut self.modal {
					Some(screen) => match screen {
						Screen::Changelog(changelog) => {
							changelog.update(changelog::Message::FadeOut);
						}
						_ => (),
					},
					None => (),
				}
				Task::perform(
					tokio::time::sleep(tokio::time::Duration::from_millis(FADE_DURATION)),
					|_| Message::ModalCleanup,
				)
			}
			Message::Nothing => Task::none(),
			Message::Progress(message) => match &mut self.modal {
				Some(screen) => match screen {
					Screen::Progress(progress) => {
						let action = progress.update(message);
						match action {
							progress::Action::Canceled => {
								self.fade.transition(false, now);
								// PERF: consider self.fade.in_progress instead of sleeping for a
								// fixed duration
								Task::perform(
									tokio::time::sleep(tokio::time::Duration::from_millis(
										FADE_DURATION,
									)),
									|_| Message::ModalCleanup,
								)
							}
							progress::Action::ExanimaLaunched => {
								Task::done(Message::ExanimaLaunched)
							}
							progress::Action::None => Task::none(),
						}
					}
					_ => Task::none(),
				},
				None => Task::none(),
			},
			Message::ScreenChanged(kind) => match kind {
				ScreenKind::Changelog => Task::none(),
				ScreenKind::Home => {
					self.screen = Screen::Home(Home::default());
					Task::none()
				}
				ScreenKind::Progress => Task::none(),
				ScreenKind::Settings => {
					self.screen = Screen::Settings(Settings::default());
					Task::none()
				}
			},
			Message::Settings(message) => match &mut self.screen {
				Screen::Settings(settings) => settings
					.update(message, &mut self.app_state)
					.map(Message::Settings),
				_ => Task::none(),
			},
			Message::StartGame(kind) => match kind {
				GameStartType::Modded => {
					log::info!("Starting modded Exanima...");
					let (progress, task) =
						Progress::new(self.app_state.settings.clone(), self.window_size * 0.8);
					self.fade.transition(true, now);
					self.modal = Some(Screen::Progress(progress));
					task.map(Message::Progress)
				}
				GameStartType::Vanilla => {
					// TODO: start vanilla exanima
					log::info!("Starting vanilla Exanima...");
					Task::none()
				}
			},
			Message::Tick => Task::none(),
		}
	}

	pub fn view(&self) -> Element<Message> {
		let now = Instant::now();

		let screen = match &self.screen {
			Screen::Home(screen) => screen.view().map(Message::Home),
			Screen::Settings(screen) => screen.view().map(Message::Settings),
			_ => unreachable!("Unsupported screen"),
		};

		let con = container(
			Row::new()
				.spacing(10.)
				.push(
					Column::new()
						.push(self.sidebar())
						.width(Length::Fixed(256.)),
				)
				.push(Column::new().push(screen).width(Length::Fill)),
		)
		.padding(12);

		if let Some(screen) = &self.modal {
			match screen {
				Screen::Changelog(changelog) => modal(
					self.fade.clone(),
					con,
					changelog.view().map(Message::Changelog),
					|| Message::ModalClosed,
				),
				Screen::Progress(progress) => modal(
					self.fade.clone(),
					con,
					progress.view().map(Message::Progress),
					|| Message::Nothing,
				),
				_ => con.into(),
			}
		} else {
			con.into()
		}
	}

	pub fn sidebar(&self) -> Element<Message> {
		container(
			Column::new()
				.push(
					Column::new().push(button(text("Home")).on_press_maybe(match self.screen {
						Screen::Home(_) => None,
						_ => Some(Message::ScreenChanged(ScreenKind::Home)),
					})),
				)
				.push(Column::new().push(button(text("Settings")).on_press_maybe(
					match self.screen {
						Screen::Settings(_) => None,
						_ => Some(Message::ScreenChanged(ScreenKind::Settings)),
					},
				)))
				.push(Column::new().push(
					button("View Changelog").on_press(Message::ModalChanged(ScreenKind::Changelog)),
				))
				.push(Column::new().height(Length::Fill))
				.push(
					Column::new().push(
						button(text("Play").size(20))
							.on_press(Message::StartGame(GameStartType::Modded))
							.width(Length::Fill),
					),
				),
		)
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
					let palette = theme::Palette::CATPPUCCIN_FRAPPE;
					return Column::new()
						.spacing(10.)
						.push(text("You're already up to date!"))
						.push(horizontal_rule(1.))
						.push(scrollable(
							markdown(
								&self.changelog,
								markdown::Settings::default(),
								markdown::Style {
									inline_code_highlight: markdown::Highlight {
										background: Background::Color(palette.background),
										border: Border::default(),
									},
									inline_code_padding: Padding::default(),
									inline_code_color: palette.text,
									link_color: palette.primary,
								},
							)
							.map(|url| Message::LinkClicked(url.to_string())),
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
							.on_press(Message::LinkClicked(release.html_url.clone()))
							.width(100.),
					)
			}
			.into(),
			GetLatestReleaseState::Error(error) => text(format!("Error: {}", error)).into(),
		}
	}

	pub fn theme(&self) -> Theme {
		Theme::CatppuccinFrappe
	}

	pub fn title(&self) -> String {
		String::from("Exanima Modding Toolkit")
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let now = Instant::now();

		// TODO: replace listen() with listen_with()
		let events = event::listen().map(Message::IcedEvent);

		let modal_fade = if self.fade.in_progress(now) {
			window::frames().map(|_| Message::Tick)
		} else {
			Subscription::none()
		};

		let modal = if let Some(screen) = &self.modal {
			match screen {
				Screen::Progress(progress) => progress.subscription().map(Message::Progress),
				_ => Subscription::none(),
			}
		} else {
			Subscription::none()
		};

		Subscription::batch([events, modal_fade, modal])
	}
}

impl Default for Emtk {
	fn default() -> Self {
		Self {
			app_state: state::AppState::default(),
			changelog: Vec::default(),
			fade: Animated::new(false)
				.duration(FADE_DURATION as f32)
				.easing(Easing::EaseOut)
				.delay(0.),
			latest_release: GetLatestReleaseState::default(),
			modal: Option::default(),
			screen: Screen::default(),
			window_size: Size::default(),
		}
	}
}

#[derive(Debug, Clone)]
pub enum GameStartType {
	Modded,
	Vanilla,
}

async fn get_latest_release() -> anyhow::Result<Release> {
	let url = "https://codeberg.org/api/v1/repos/ExanimaModding/Toolkit/releases/latest";

	let release: Release = ureq::get(url).call()?.into_json()?;

	Ok(release)
}
