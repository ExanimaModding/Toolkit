mod constants;
mod screen;
mod state;
mod theme;
mod widget;

use std::{path::PathBuf, time::Instant};

use constants::FADE_DURATION;
use iced::{
	event,
	widget::{
		button, container, horizontal_rule, markdown, scrollable, text, vertical_space, Column, Row,
	},
	window, Background, Border, Color, Element, Length, Padding, Size, Subscription, Task, Theme,
};
use lilt::{Animated, Easing};
use screen::{
	changelog::{self, Changelog},
	explorer::{self, Explorer},
	home::{self, Home},
	progress::{self, Progress},
	settings::{self, Settings},
	Screen, ScreenKind,
};
use widget::modal::modal;

// TODO: animate scrolling in scrollbars
static ICON: &[u8] = include_bytes!("../../../../assets/images/corro.ico");

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

pub struct Emtk {
	app_state: state::AppState,
	changelog: Vec<markdown::Item>,
	developer_enabled: bool,
	explain_enabled: bool,
	fade: Animated<bool, Instant>,
	latest_release: GetLatestReleaseState,
	modal: Option<Screen>,
	screen: Screen,
	theme: Theme,
	window_size: Size,
}

#[derive(Debug, Clone)]
pub enum Message {
	Changelog(changelog::Message),
	ExanimaLaunched,
	Explorer(explorer::Message),
	GetLatestRelease(GetLatestReleaseState),
	Home(home::Message),
	LinkClicked(String),
	ModalChanged(ScreenKind),
	ModalCleanup,
	ModalClosed,
	Nothing,
	Progress(progress::Message),
	ScreenChanged(ScreenKind),
	Settings(settings::Message),
	SizeChanged(Size),
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
						return Task::batch([task.map(Message::Changelog), action]);
					}
					_ => (),
				},
				None => (),
			},
			// TODO: launch exanima
			// crate::launch_exanima();
			Message::ExanimaLaunched => log::info!("Launching exanima..."),
			Message::Explorer(message) => match &mut self.screen {
				Screen::Explorer(explorer) => {
					return explorer.update(message).map(Message::Explorer);
				}
				_ => (),
			},
			Message::GetLatestRelease(state) => match state {
				GetLatestReleaseState::NotStarted => {
					log::info!("Checking for updates...");
					self.latest_release = GetLatestReleaseState::Loading;
					return Task::future(get_latest_release()).map(|result| match result {
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
					});
				}
				GetLatestReleaseState::Loading => (),
				GetLatestReleaseState::Loaded(release) => {
					self.changelog =
						markdown::parse(&format!("[View in browser]({})\n", release.html_url))
							.collect();

					let mut changelog: Vec<_> = markdown::parse(&release.body).collect();

					self.changelog.append(&mut changelog);
					log::info!("Latest release: {}", release.tag_name);
					self.latest_release = GetLatestReleaseState::Loaded(release);
				}
				GetLatestReleaseState::Error(error) => {
					log::error!("Error checking for updates: {}", error);
					self.latest_release = GetLatestReleaseState::Error(error);
				}
			},
			Message::Home(message) => match &mut self.screen {
				Screen::Home(home) => {
					return home.update(message, &mut self.app_state).map(Message::Home)
				}
				_ => (),
			},
			Message::LinkClicked(url) => {
				log::info!("Opening URL: {}", url);
				open::that(url).unwrap();
			}
			Message::ModalChanged(kind) => match kind {
				ScreenKind::Changelog => {
					self.fade.transition(true, now);
					self.modal = Some(Screen::Changelog(Changelog::new(
						self.changelog.clone(),
						self.latest_release.clone(),
						Some(self.window_size * 0.8),
						self.theme.to_owned(),
					)));
				}
				_ => (),
			},
			Message::ModalCleanup => self.modal = None,
			Message::ModalClosed => {
				self.fade.transition(false, now);
				match &mut self.modal {
					Some(screen) => match screen {
						Screen::Changelog(changelog) => {
							let (_task, _action) = changelog.update(changelog::Message::FadeOut);
						}
						_ => (),
					},
					None => (),
				}
				return Task::perform(
					tokio::time::sleep(tokio::time::Duration::from_millis(FADE_DURATION)),
					|_| Message::ModalCleanup,
				);
			}
			Message::Nothing => (),
			Message::Progress(message) => match &mut self.modal {
				Some(screen) => match screen {
					Screen::Progress(progress) => {
						let action = progress.update(message);
						match action {
							progress::Action::Canceled => {
								self.fade.transition(false, now);
								// PERF: consider self.fade.in_progress instead of sleeping for a
								// fixed duration
								return Task::perform(
									tokio::time::sleep(tokio::time::Duration::from_millis(
										FADE_DURATION,
									)),
									|_| Message::ModalCleanup,
								);
							}
							progress::Action::ExanimaLaunched => {
								let (task, _action) = match &mut self.screen {
									Screen::Settings(settings) => settings.update(
										settings::Message::CacheChecked,
										&mut self.app_state,
									),
									_ => (Task::none(), settings::Action::None),
								};
								return Task::batch([
									Task::done(Message::ExanimaLaunched),
									task.map(Message::Settings),
								]);
							}
							progress::Action::None => (),
						}
					}
					_ => (),
				},
				None => (),
			},
			Message::ScreenChanged(kind) => match kind {
				ScreenKind::Changelog => (),
				ScreenKind::Explorer => {
					let exanima_exe =
						PathBuf::from(self.app_state.settings.exanima_exe.clone().unwrap());
					// TODO: redundant code taken from crate::gui::screen::progress::load_mods()
					let exanima_path = exanima_exe
						.parent()
						.expect("error while getting parent directory of exanima exe");

					let exanima_rpks: Vec<PathBuf> = exanima_path
						.read_dir()
						.expect("error while reading exanima directory")
						.flatten()
						.filter_map(|entry| {
							let path = entry.path();
							let file_name = path
								.file_name()
								.expect("error while reading file name")
								.to_str()
								.expect("error while getting file name");
							if path.is_dir() || !file_name.ends_with(".rpk") {
								None
							} else {
								Some(path)
							}
						})
						.collect();

					self.screen = Screen::Explorer(Explorer::new(exanima_rpks))
				}
				ScreenKind::Home => self.screen = Screen::Home(Home::default()),
				ScreenKind::Progress => (),
				ScreenKind::Settings => {
					let (settings, task) = Settings::new(
						self.developer_enabled,
						self.explain_enabled,
						self.theme.to_owned(),
					);
					self.screen = Screen::Settings(settings);
					return task.map(Message::Settings);
				}
			},
			Message::Settings(message) => match &mut self.screen {
				Screen::Settings(settings) => {
					let (task, action) = settings.update(message, &mut self.app_state);
					let action = match action {
						settings::Action::DeveloperToggled(developer_enabled) => {
							self.developer_enabled = developer_enabled;
							Task::none()
						}
						settings::Action::ExplainToggled(explain_enabled) => {
							self.explain_enabled = explain_enabled;
							Task::none()
						}
						settings::Action::ThemeChanged(theme) => {
							self.theme = theme;
							Task::none()
						}
						settings::Action::ViewChangelog => {
							Task::done(Message::ModalChanged(ScreenKind::Changelog))
						}
						settings::Action::None => Task::none(),
					};
					return Task::batch([task.map(Message::Settings), action]);
				}
				_ => (),
			},
			Message::SizeChanged(size) => {
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
						return task.map(Message::Changelog);
					}
					Screen::Progress(progress) => {
						let width = size.width * 0.8;
						let size = Size::new(width, 0.);
						let _action = progress.update(progress::Message::SizeChanged(size));
					}
					_ => (),
				}
			}
			Message::StartGame(kind) => match kind {
				GameStartType::Modded => {
					log::info!("Starting modded Exanima...");
					let (progress, task) =
						Progress::new(self.app_state.settings.clone(), self.window_size * 0.8);
					self.fade.transition(true, now);
					self.modal = Some(Screen::Progress(progress));
					return task.map(Message::Progress);
				}
				// TODO: start vanilla exanima
				GameStartType::Vanilla => log::info!("Starting vanilla Exanima..."),
			},
			Message::Tick => (),
		};

		Task::none()
	}

	pub fn view(&self) -> Element<Message> {
		let now = Instant::now();

		let screen = match &self.screen {
			Screen::Home(home) => home.view().map(Message::Home),
			Screen::Explorer(explorer) => explorer.view().map(Message::Explorer),
			Screen::Settings(settings) => settings.view().map(Message::Settings),
			_ => unreachable!("Unsupported screen"),
		};

		let con = container(
			Row::new()
				.push(
					Column::new()
						.push(self.sidebar())
						.width(Length::Fixed(256.)),
				)
				.push(Column::new().push(screen).width(Length::Fill)),
		)
		.padding(12);

		let con = if let Some(screen) = &self.modal {
			match screen {
				Screen::Changelog(changelog) => {
					let changelog_view = changelog.view().map(Message::Changelog);
					let changelog_view = if self.explain_enabled {
						changelog_view.explain(Color::BLACK)
					} else {
						changelog_view
					};
					modal(self.fade.clone(), con, changelog_view, || {
						Message::ModalClosed
					})
				}
				Screen::Progress(progress) => {
					let progress_view = progress.view().map(Message::Progress);
					let progress_view = if self.explain_enabled {
						progress_view.explain(Color::BLACK)
					} else {
						progress_view
					};
					modal(self.fade.clone(), con, progress_view, || Message::Nothing)
				}
				_ => con.into(),
			}
		} else {
			con.into()
		};

		if self.explain_enabled {
			con.explain(Color::BLACK)
		} else {
			con
		}
	}

	pub fn sidebar(&self) -> Element<Message> {
		container(
			Column::new()
				.push(
					Column::new().push(
						button(text("Home"))
							.on_press_maybe(match self.screen {
								Screen::Home(_) => None,
								_ => Some(Message::ScreenChanged(ScreenKind::Home)),
							})
							.width(Length::Fill)
							.style(theme::transparent_button),
					),
				)
				.push(
					Column::new().push(
						button(text("Explorer"))
							.on_press_maybe(match self.screen {
								Screen::Explorer(_) => None,
								_ => Some(Message::ScreenChanged(ScreenKind::Explorer)),
							})
							.width(Length::Fill)
							.style(theme::transparent_button),
					),
				)
				.push(
					Column::new().push(
						button(text("Settings"))
							.on_press_maybe(match self.screen {
								Screen::Settings(_) => None,
								_ => Some(Message::ScreenChanged(ScreenKind::Settings)),
							})
							.width(Length::Fill)
							.style(theme::transparent_button),
					),
				)
				.push(vertical_space())
				.push(
					Column::new().push(
						button(text("Play").size(20))
							.on_press(Message::StartGame(GameStartType::Modded))
							.width(Length::Fill)
							.style(theme::button),
					),
				)
				.spacing(1),
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
					return Column::new()
						.spacing(10.)
						.push(text("You're already up to date!"))
						.push(horizontal_rule(1.))
						.push(scrollable(
							markdown(
								&self.changelog,
								markdown::Settings::default(),
								markdown::Style::from_palette(self.theme.palette()),
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
							.width(100.)
							.style(theme::button),
					)
			}
			.into(),
			GetLatestReleaseState::Error(error) => text(format!("Error: {}", error)).into(),
		}
	}

	pub fn theme(&self) -> Theme {
		self.theme.to_owned()
	}

	pub fn title(&self) -> String {
		String::from("Exanima Modding Toolkit")
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let now = Instant::now();

		let events = event::listen_with(|event, _status, _id| match event {
			iced::Event::Window(event) => match event {
				window::Event::Resized(size) => Some(Message::SizeChanged(size)),
				_ => None,
			},
			_ => None,
		});

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
			developer_enabled: bool::default(),
			explain_enabled: bool::default(),
			fade: Animated::new(false)
				.duration(FADE_DURATION as f32)
				.easing(Easing::EaseOut)
				.delay(0.),
			latest_release: GetLatestReleaseState::default(),
			modal: Option::default(),
			screen: Screen::default(),
			theme: Theme::CatppuccinFrappe,
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
