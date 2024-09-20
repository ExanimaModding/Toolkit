mod constants;
mod screen;
mod theme;
mod widget;

use std::{
	collections::HashMap,
	fs,
	io::Read,
	path::{Path, PathBuf},
	time::Instant,
};

use constants::FADE_DURATION;
use emf_types::config::PluginConfig;
use iced::{
	event,
	widget::{
		button, container, horizontal_rule, markdown, scrollable, svg, text, vertical_space,
		Column, Row,
	},
	window, Alignment, Color, Element, Length, Size, Subscription, Task, Theme,
};
use lilt::{Animated, Easing};
use screen::{
	changelog::{self, Changelog},
	explorer::{self, Explorer},
	mods::{self, Mods},
	progress::{self, Progress},
	settings::{self, Settings},
	Screen, ScreenKind,
};
use strum::{EnumIter, IntoEnumIterator};
use widget::modal::modal;

use crate::config::{self, Config};

// TODO: animate scrolling in scrollbars
static ICON: &[u8] = include_bytes!("../../../../assets/images/corro.ico");

#[derive(Debug, Hash, PartialEq, Eq, EnumIter)]
pub enum Icon {
	ArrowLeft,
	CircleAlert,
	Folder,
	Layers,
	Menu,
	Play,
	Settings,
	SquareArrowOutUpRight,
}

impl Icon {
	fn bytes(&self) -> &'static [u8] {
		match self {
			Icon::ArrowLeft => include_bytes!("../../../../assets/images/arrow-left.svg"),
			Icon::CircleAlert => include_bytes!("../../../../assets/images/circle-alert.svg"),
			Icon::Folder => include_bytes!("../../../../assets/images/folder.svg"),
			Icon::Layers => include_bytes!("../../../../assets/images/layers-3.svg"),
			Icon::Menu => include_bytes!("../../../../assets/images/menu.svg"),
			Icon::Play => include_bytes!("../../../../assets/images/play.svg"),
			Icon::Settings => include_bytes!("../../../../assets/images/settings.svg"),
			Icon::SquareArrowOutUpRight => {
				include_bytes!("../../../../assets/images/square-arrow-out-up-right.svg")
			}
		}
	}
}

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
	changelog: Vec<markdown::Item>,
	config: Config,
	fade: Animated<bool, Instant>,
	icons: HashMap<Icon, svg::Handle>,
	latest_release: GetLatestReleaseState,
	modal: Option<Screen>,
	screen: Screen,
	window_size: Size,
}

#[derive(Debug, Clone)]
pub enum Message {
	Changelog(changelog::Message),
	ExanimaLaunched,
	Explorer(explorer::Message),
	GetLatestRelease(GetLatestReleaseState),
	Mods(mods::Message),
	LinkClicked(String),
	ModalChanged(ScreenKind),
	ModalCleanup,
	ModalClosed,
	Nothing,
	Progress(progress::Message),
	ScreenChanged(ScreenKind),
	Settings(settings::Message),
	ConfigChanged(Config),
	SizeChanged(Size),
	StartGame,
	Tick,
}

impl Emtk {
	pub fn new() -> (Self, Task<Message>) {
		let config_path = dirs::config_dir().unwrap().join("Exanima Modding Toolkit");
		if !config_path.is_dir() {
			fs::create_dir_all(&config_path).unwrap();
		}
		let config_path = config_path.join("config.ron");
		let default_config = Config {
			exanima_exe: Option::default(),
			launcher: Some(config::Launcher::default()),
			load_order: Vec::new(),
		};
		let config = if config_path.is_file() {
			let mut contents = String::new();
			fs::File::open(&config_path)
				.unwrap()
				.read_to_string(&mut contents)
				.unwrap();
			// TODO: attempt to migrate old settings on error result
			match ron::from_str::<Config>(&contents) {
				Ok(config) => config,
				Err(_) => default_config,
			}
		} else {
			default_config
		};
		let task_configure = if config.exanima_exe.is_none() {
			// TODO: attempt to find Exanima.exe via Steam
			Task::done(Message::ModalChanged(ScreenKind::Settings))
		} else {
			Task::none()
		};

		let mut icons = HashMap::new();
		for icon in Icon::iter() {
			let bytes = icon.bytes();
			icons.insert(icon, svg::Handle::from_memory(bytes));
		}

		let (mods, mods_action) = Mods::new(config.clone());
		let mods_task = match mods_action {
			mods::Action::ConfigChanged(config) => Task::done(Message::ConfigChanged(config)),
			mods::Action::Run(task) => task.map(Message::Mods),
			mods::Action::None => Task::none(),
		};

		let emtk = Self {
			config,
			icons,
			screen: Screen::Mods(mods),
			..Default::default()
		};
		(
			emtk,
			// TODO: refactor
			// Task::batch([
			// 	Task::done(screen::settings::Message::default()).map(Message::Settings),
			// 	Task::done(screen::home::Message::LoadSettings(settings.clone()))
			// 		.map(Message::Home),
			// ]),
			Task::batch([
				mods_task,
				task_configure,
				Task::done(Message::GetLatestRelease(GetLatestReleaseState::NotStarted)),
				Task::done(Message::ScreenChanged(ScreenKind::Mods)),
			]),
		)
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		let now = Instant::now();

		match message {
			Message::Changelog(message) => {
				if let Some(Screen::Changelog(changelog)) = &mut self.modal {
					let (task, action) = changelog.update(message);
					let action = match action {
						changelog::Action::LinkClicked(url) => {
							Task::done(Message::LinkClicked(url))
						}
						changelog::Action::None => Task::none(),
					};
					return Task::batch([task.map(Message::Changelog), action]);
				}
			}
			// TODO: launch exanima
			// crate::launch_exanima();
			Message::ExanimaLaunched => log::info!("Launching exanima..."),
			Message::Explorer(message) => {
				if let Screen::Explorer(explorer) = &mut self.screen {
					return explorer.update(message).map(Message::Explorer);
				}
			}
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
			Message::Mods(message) => {
				if let Screen::Mods(mods) = &mut self.screen {
					return match mods.update(message) {
						mods::Action::ConfigChanged(config) => {
							Task::done(Message::ConfigChanged(config))
						}
						mods::Action::Run(task) => task.map(Message::Mods),
						mods::Action::None => Task::none(),
					};
				}
			}
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
						self.theme(),
					)))
				}
				ScreenKind::Settings => {
					self.fade.transition(true, now);
					let (settings, task) = Settings::new(
						self.config.clone(),
						self.theme(),
						Some(self.window_size * 0.8),
					);
					self.modal = Some(Screen::Settings(settings));
					return task.map(Message::Settings);
				}
				_ => (),
			},
			Message::ModalCleanup => self.modal = None,
			Message::ModalClosed => {
				let Some(screen) = &mut self.modal else {
					return Task::none();
				};
				self.fade.transition(false, now);
				match screen {
					Screen::Changelog(changelog) => {
						let (_task, _action) = changelog.update(changelog::Message::FadeOut);
					}
					Screen::Settings(settings) => {
						settings.update(settings::Message::FadeOut);
					}
					_ => (),
				}
				return Task::perform(
					tokio::time::sleep(tokio::time::Duration::from_millis(FADE_DURATION)),
					|_| Message::ModalCleanup,
				);
			}
			Message::Nothing => (),
			Message::Progress(message) => {
				if let Some(Screen::Progress(progress)) = &mut self.modal {
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
							let task = match &mut self.screen {
								Screen::Settings(settings) => {
									match settings.update(settings::Message::CacheChecked) {
										settings::Action::Run(task) => task,
										_ => unreachable!("This is a bug. Please report this."),
									}
								}
								_ => unreachable!("This is a bug. Please report this."),
							};
							return Task::batch([
								Task::done(Message::ExanimaLaunched),
								task.map(Message::Settings),
							]);
						}
						progress::Action::None => (),
					}
				}
			}
			Message::ScreenChanged(kind) => match kind {
				ScreenKind::Changelog => (),
				ScreenKind::Explorer => {
					let exanima_exe = PathBuf::from(self.config.exanima_exe.clone().unwrap());
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
				ScreenKind::Mods => {
					let (mods, mods_action) = Mods::new(self.config.clone());
					self.screen = Screen::Mods(mods);
					return match mods_action {
						mods::Action::ConfigChanged(config) => {
							Task::done(Message::ConfigChanged(config))
						}
						mods::Action::Run(task) => task.map(Message::Mods),
						mods::Action::None => Task::none(),
					};
				}
				ScreenKind::Progress => (),
				ScreenKind::Settings => {
					let (settings, task) = Settings::new(self.config.clone(), self.theme(), None);
					self.screen = Screen::Settings(settings);
					return task.map(Message::Settings);
				}
			},
			Message::Settings(message) => {
				let settings = if let Some(Screen::Settings(settings)) = &mut self.modal {
					settings
				} else if let Screen::Settings(settings) = &mut self.screen {
					settings
				} else {
					return Task::none();
				};

				return match settings.update(message) {
					settings::Action::CloseModal => {
						if let Some(Screen::Settings(_settings)) = &mut self.modal {
							Task::done(Message::ModalClosed)
						} else {
							Task::none()
						}
					}
					settings::Action::ConfigChanged(settings) => {
						Task::done(Message::ConfigChanged(settings))
					}
					settings::Action::Run(task) => task.map(Message::Settings),
					settings::Action::ViewChangelog => {
						Task::done(Message::ModalChanged(ScreenKind::Changelog))
					}
					settings::Action::None => Task::none(),
				};
			}
			Message::ConfigChanged(config) => {
				let config_path = dirs::config_dir().unwrap().join("Exanima Modding Toolkit");
				if !config_path.is_dir() {
					fs::create_dir_all(&config_path).unwrap();
				}
				let config_path = config_path.join("config.ron");
				let content =
					ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default()).unwrap();
				fs::write(config_path, content).unwrap();
				self.config = config;
				match &mut self.screen {
					Screen::Mods(mods) => {
						mods.update(mods::Message::ConfigRefetched(self.config.clone()));
					}
					Screen::Settings(settings) => {
						match settings
							.update(settings::Message::ConfigRefetched(self.config.clone()))
						{
							settings::Action::None => (),
							_ => unreachable!("This is a bug. Please report this."),
						};
					}
					_ => (),
				}
			}
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
					Screen::Settings(settings) => {
						let width = size.width * 0.8;
						let height = size.height * 0.8;
						let size = Size::new(width, height);
						settings.update(settings::Message::SizeChanged(size));
					}
					_ => (),
				}
			}
			Message::StartGame => {
				log::info!("Starting Exanima...");
				let (progress, task) = Progress::new(self.config.clone(), self.window_size * 0.8);
				self.fade.transition(true, now);
				self.modal = Some(Screen::Progress(progress));
				return task.map(Message::Progress);
			}
			Message::Tick => (),
		};

		Task::none()
	}

	pub fn view(&self) -> Element<Message> {
		let explain_color = if self.theme().extended_palette().is_dark {
			Color::WHITE
		} else {
			Color::BLACK
		};

		let screen = match &self.screen {
			Screen::Mods(home) => home.view(&self.icons).map(Message::Mods),
			Screen::Explorer(explorer) => explorer.view(&self.icons).map(Message::Explorer),
			Screen::Settings(settings) => settings.view(&self.icons).map(Message::Settings),
			_ => unreachable!("Unsupported screen"),
		};

		let con = container(
			Row::new()
				.push(
					Column::new()
						.push(self.sidebar())
						.width(Length::Fixed(216.)),
				)
				.push(Column::new().push(screen).width(Length::Fill)),
		)
		.padding(12);

		let con = if let Some(screen) = &self.modal {
			match screen {
				Screen::Changelog(changelog) => {
					let changelog_view = changelog.view().map(Message::Changelog);
					let changelog_view = if self.config.launcher.as_ref().unwrap().explain {
						changelog_view.explain(explain_color)
					} else {
						changelog_view
					};
					modal(self.fade.clone(), con, changelog_view, || {
						Message::ModalClosed
					})
				}
				Screen::Progress(progress) => {
					let progress_view = progress.view().map(Message::Progress);
					let progress_view = if self.config.launcher.as_ref().unwrap().explain {
						progress_view.explain(explain_color)
					} else {
						progress_view
					};
					modal(self.fade.clone(), con, progress_view, || Message::Nothing)
				}
				Screen::Settings(settings) => {
					let settings_view = settings.view(&self.icons).map(Message::Settings);
					let settings_view = if self.config.launcher.as_ref().unwrap().explain {
						settings_view.explain(explain_color)
					} else {
						settings_view
					};
					modal(self.fade.clone(), con, settings_view, || Message::Nothing)
				}
				_ => con.into(),
			}
		} else {
			con.into()
		};

		if self.config.launcher.as_ref().unwrap().explain {
			con.explain(explain_color)
		} else {
			con
		}
	}

	pub fn sidebar(&self) -> Element<Message> {
		container(
			Column::new()
				.push(
					Column::new().push(
						button(
							Row::new()
								.push(
									svg(self.icons.get(&Icon::Layers).unwrap().clone())
										.width(Length::Shrink)
										.style({
											if let Screen::Mods(_mods) = &self.screen {
												theme::svg_button
											} else {
												theme::svg
											}
										}),
								)
								.push(text("Mods").size(18))
								.spacing(12),
						)
						.on_press_maybe(match self.screen {
							Screen::Mods(_) => None,
							_ => Some(Message::ScreenChanged(ScreenKind::Mods)),
						})
						.width(Length::Fill)
						.style(theme::transparent_button),
					),
				)
				.push(
					Column::new().push(
						button(
							Row::new()
								.push(
									svg(self.icons.get(&Icon::Folder).unwrap().clone())
										.width(Length::Shrink)
										.style({
											if let Screen::Explorer(_explorer) = &self.screen {
												theme::svg_button
											} else {
												theme::svg
											}
										}),
								)
								.push(text("Explorer").size(18))
								.spacing(12),
						)
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
						button(
							Row::new()
								.push(
									svg(self.icons.get(&Icon::Settings).unwrap().clone())
										.width(Length::Shrink)
										.style({
											if let Screen::Settings(_settings) = &self.screen {
												theme::svg_button
											} else {
												theme::svg
											}
										}),
								)
								.push(text("Settings").size(18))
								.spacing(12),
						)
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
					button(
						container(
							Row::new()
								.push(
									svg(self.icons.get(&Icon::Play).unwrap().clone())
										.width(Length::Shrink)
										.height(Length::Fixed(36.))
										.style(theme::svg_button),
								)
								.push(text("Play").size(28))
								.spacing(6),
						)
						.width(Length::Fill)
						.align_x(Alignment::Center),
					)
					.on_press(Message::StartGame)
					.style(button::primary),
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
								markdown::Style::from_palette(self.theme().palette()),
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
							.style(button::primary),
					)
			}
			.into(),
			GetLatestReleaseState::Error(error) => text(format!("Error: {}", error)).into(),
		}
	}

	pub fn theme(&self) -> Theme {
		match self.config.launcher.as_ref().unwrap().theme.as_str() {
			"light" => Theme::Light,
			"dark" => Theme::Dark,
			"dracula" => Theme::Dracula,
			"nord" => Theme::Nord,
			"solarized_light" => Theme::SolarizedLight,
			"solarized_dark" => Theme::SolarizedDark,
			"gruvbox_light" => Theme::GruvboxLight,
			"gruvbox_dark" => Theme::GruvboxDark,
			"catppuccin_latte" => Theme::CatppuccinLatte,
			"catppuccin_frappe" => Theme::CatppuccinFrappe,
			"catppuccin_macchiato" => Theme::CatppuccinMacchiato,
			"catppuccin_mocha" => Theme::CatppuccinMocha,
			"tokyo_night" => Theme::TokyoNight,
			"tokyo_night_storm" => Theme::TokyoNightStorm,
			"tokyo_night_light" => Theme::TokyoNightLight,
			"kanagawa_wave" => Theme::KanagawaWave,
			"kanagawa_dragon" => Theme::KanagawaDragon,
			"kanagawa_lotus" => Theme::KanagawaLotus,
			"moonfly" => Theme::Moonfly,
			"nightfly" => Theme::Nightfly,
			"oxocarbon" => Theme::Oxocarbon,
			"ferra" => Theme::Ferra,
			// TODO: handle Theme::Custom()
			_ => Theme::Light,
		}
	}

	pub fn title(&self) -> String {
		String::from("Exanima Modding Toolkit")
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let now = Instant::now();

		let events = event::listen_with(|event, _status, _id| {
			if let iced::Event::Window(window::Event::Resized(size)) = event {
				Some(Message::SizeChanged(size))
			} else {
				None
			}
		});

		let screen = match &self.screen {
			Screen::Explorer(explorer) => explorer.subscription().map(Message::Explorer),
			Screen::Settings(settings) => settings.subscription().map(Message::Settings),
			_ => Subscription::none(),
		};

		let modal_fade = if self.fade.in_progress(now) {
			window::frames().map(|_| Message::Tick)
		} else {
			Subscription::none()
		};

		let modal = if let Some(Screen::Settings(settings)) = &self.modal {
			settings.subscription().map(Message::Settings)
		} else {
			Subscription::none()
		};

		Subscription::batch([events, screen, modal_fade, modal])
	}
}

impl Default for Emtk {
	fn default() -> Self {
		Self {
			changelog: Vec::default(),
			config: Config::default(),
			fade: Animated::new(false)
				.duration(FADE_DURATION as f32)
				.easing(Easing::EaseOut)
				.delay(0.),
			icons: HashMap::default(),
			latest_release: GetLatestReleaseState::default(),
			modal: Option::default(),
			screen: Screen::default(),
			window_size: Size::default(),
		}
	}
}

async fn get_latest_release() -> anyhow::Result<Release> {
	let url = "https://codeberg.org/api/v1/repos/ExanimaModding/Toolkit/releases/latest";

	let release: Release = ureq::get(url).call()?.into_json()?;

	Ok(release)
}

/// `path` argument must be the path to Exanima.exe
pub fn load_order(path: &Path) -> Vec<(String, bool)> {
	let mods_path = path.parent().unwrap().join("mods");
	if !mods_path.is_dir() {
		return Vec::new();
	}
	let mut load_order = Vec::new();
	for entry in mods_path.read_dir().unwrap().flatten() {
		let entry_path = entry.path();

		for entry in entry_path.read_dir().unwrap().flatten() {
			let entry_path = entry.path();

			if entry.file_name().to_str().unwrap() != "config.toml" {
				continue;
			}
			let mut contents = String::new();
			fs::File::open(&entry_path)
				.unwrap()
				.read_to_string(&mut contents)
				.unwrap();
			let mod_config: PluginConfig = match toml::from_str(&contents) {
				Ok(plugin_config) => plugin_config,
				Err(_) => continue,
			};
			load_order.push((mod_config.plugin.id, false));
		}
	}
	load_order
}

pub fn config_by_id(path: &Path, mod_id: &str) -> Option<PluginConfig> {
	let mods_path = path.parent().unwrap().join("mods");
	if !mods_path.is_dir() {
		return None;
	}
	for entry in mods_path.read_dir().unwrap().flatten() {
		let entry_path = entry.path();

		for entry in entry_path.read_dir().unwrap().flatten() {
			let entry_path = entry.path();

			if entry.file_name().to_str().unwrap() != "config.toml" {
				continue;
			}
			let mut contents = String::new();
			fs::File::open(&entry_path)
				.unwrap()
				.read_to_string(&mut contents)
				.unwrap();
			let mod_config: PluginConfig = match toml::from_str(&contents) {
				Ok(plugin_config) => plugin_config,
				Err(_) => continue,
			};
			if mod_config.plugin.id == mod_id {
				return Some(mod_config);
			}
		}
	}

	None
}

pub fn path_by_id(path: &Path, mod_id: &str) -> Option<PathBuf> {
	let mods_path = path.parent().unwrap().join("mods");
	if !mods_path.is_dir() {
		return None;
	}
	for entry in mods_path.read_dir().unwrap().flatten() {
		let mod_path = entry.path();

		for entry in mod_path.read_dir().unwrap().flatten() {
			let entry_path = entry.path();

			if entry.file_name().to_str().unwrap() != "config.toml" {
				continue;
			}
			let mut contents = String::new();
			fs::File::open(&entry_path)
				.unwrap()
				.read_to_string(&mut contents)
				.unwrap();
			let mod_config: PluginConfig = match toml::from_str(&contents) {
				Ok(plugin_config) => plugin_config,
				Err(_) => continue,
			};
			if mod_config.plugin.id == mod_id {
				return Some(mod_path);
			}
		}
	}

	None
}

pub fn missing_mods(order: &[(String, bool)], path: &Path) -> Vec<(String, bool)> {
	order
		.iter()
		.filter_map(|(mod_id, enabled)| {
			if load_order(path)
				.iter()
				.any(|(maybe_mod_id, _enabled)| mod_id == maybe_mod_id)
			{
				None
			} else {
				Some((mod_id.clone(), *enabled))
			}
		})
		.collect::<Vec<_>>()
}
