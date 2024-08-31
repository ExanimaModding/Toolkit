mod constants;
mod screen;
mod state;
mod widget;

use crate::config::AppSettings;
use exparser::{deku::prelude::*, Format};
use iced::{
	event,
	futures::{channel::mpsc::Sender, SinkExt, Stream},
	widget::{
		button, container, horizontal_rule, markdown, progress_bar, scrollable, text, Column, Row,
	},
	window, Element, Length, Padding, Size, Subscription, Task, Theme,
};
use screen::{
	changelog::{self, Changelog},
	home::{self, Home},
	settings::{self, Settings},
	Screen, ScreenKind,
};
use std::{fs, io, path::PathBuf};
use widget::modal::modal;

static ICON: &[u8] = include_bytes!("../../../../assets/images/corro.ico");

pub(crate) async fn start_gui() -> iced::Result {
	let image = image::load_from_memory(ICON).unwrap();
	let icon =
		iced::window::icon::from_rgba(image.as_bytes().to_vec(), image.height(), image.width())
			.unwrap();

	iced::application("Exanima Modding Toolkit", Emtk::update, Emtk::view)
		.theme(Emtk::theme)
		.window(iced::window::Settings {
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

#[derive(Debug, Default, Clone)]
pub struct Emtk {
	app_state: state::AppState,
	changelog: Vec<markdown::Item>,
	game_start_state: GameStartState,
	latest_release: GetLatestReleaseState,
	modal: Option<Screen>,
	screen: Screen,
	window_size: Size,
}

#[derive(Debug, Clone)]
pub enum Message {
	Changelog(changelog::Message),
	Event(Event),
	GetLatestRelease(GetLatestReleaseState),
	Home(home::Message),
	IcedEvent(iced::Event),
	ModalChanged(ScreenKind),
	ModalClosed,
	ScreenChanged(ScreenKind),
	Settings(settings::Message),
	StartGame(GameStartType),
	LinkClicked(String),
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
			Message::Event(event) => match event {
				Event::ExanimaLaunched(state) => {
					self.game_start_state = state;
					// TODO: launch exanima
					// crate::launch_exanima();
					log::info!("Launching exanima...");
					Task::none()
				}
				Event::ProgressUpdated(bar) => {
					self.game_start_state = GameStartState::Loading(bar);
					Task::none()
				}
			},
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
							_ => Task::none(),
						}
					}
					_ => Task::none(),
				},
				_ => Task::none(),
			},
			Message::ModalChanged(kind) => match kind {
				ScreenKind::Changelog => {
					self.modal = Some(Screen::Changelog(Changelog::new(
						self.changelog.clone(),
						self.latest_release.clone(),
						Some(self.window_size * 0.8),
					)));
					Task::none()
				}
				_ => Task::none(),
			},
			Message::ModalClosed => {
				self.modal = None;
				Task::none()
			}
			Message::ScreenChanged(kind) => match kind {
				ScreenKind::Changelog => Task::none(),
				ScreenKind::Home => {
					self.screen = Screen::Home(Home::default());
					Task::none()
				}
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
					self.game_start_state = GameStartState::Loading(ProgressBar::default());
					log::info!("Starting modded Exanima...");
					Task::stream(load_mods(self.app_state.settings.clone())).map(Message::Event)
				}
				GameStartType::Vanilla => {
					// TODO: start vanilla exanima
					self.game_start_state = GameStartState::Loading(ProgressBar::default());
					log::info!("Starting vanilla Exanima...");
					Task::none()
				}
			},
			Message::LinkClicked(url) => {
				log::info!("Opening URL: {}", url);
				open::that(url).unwrap();
				Task::none()
			}
		}
	}

	pub fn view(&self) -> Element<Message> {
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
		.padding(Padding::new(12.0));

		if let Some(screen) = &self.modal {
			match screen {
				Screen::Changelog(changelog) => {
					modal(con, changelog.view().map(Message::Changelog), || {
						Message::ModalClosed
					})
				}
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
				.push(Column::new().push(text("Sidebar!")).height(Length::Fill))
				.push(Column::new().push(self.play_buttons())),
		)
		.into()
	}

	fn play_buttons(&self) -> Element<Message> {
		let play_button = button(text("Play").size(20)).width(Length::FillPortion(7));

		match self.game_start_state {
			GameStartState::NotStarted => Row::new()
				// TODO: use on_press_maybe
				.push(play_button.on_press(Message::StartGame(GameStartType::Modded)))
				.into(),
			_ => Row::new().push(play_button).into(),
		}
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
					let palette = iced::theme::Palette::CATPPUCCIN_FRAPPE;
					return Column::new()
						.spacing(10.)
						.push(text("You're already up to date!"))
						.push(horizontal_rule(1.))
						// TODO: make changelog a modal
						.push(scrollable(
							markdown(
								&self.changelog,
								markdown::Settings::default(),
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

	pub fn progress_bars(&self) -> Element<Message> {
		Column::new()
			.push_maybe(
				if let GameStartState::Loading(progress) = &self.game_start_state {
					let progress_col = Column::new().push(
						progress_bar(
							0.0..=progress.rpks.len() as f32,
							(progress.rpk_step + 1) as f32,
						)
						.height(Length::Fixed(10.0)),
					);

					let rpk_row = Row::new().push(
						text(format!(
							"Rpks: {} / {}",
							progress.rpk_step + 1,
							progress.rpks.len(),
						))
						.width(Length::Fill),
					);
					let rpk_name = progress.rpks.get(progress.rpk_step);
					let rpk_row = if let Some(name) = rpk_name {
						rpk_row.push(text(name))
					} else {
						rpk_row
					};
					let progress_col = progress_col.push(rpk_row);

					let progress_col = progress_col.push(
						progress_bar(
							0.0..=progress.mods.len() as f32,
							(progress.mod_step + 1) as f32,
						)
						.height(Length::Fixed(10.0)),
					);

					let mod_row = Row::new().push(
						text(format!(
							"Mods: {} / {}",
							progress.mod_step + 1,
							progress.mods.len(),
						))
						.width(Length::Fill),
					);
					let mod_name = progress.mods.get(progress.mod_step);
					let mod_row = if let Some(name) = mod_name {
						mod_row.push(text(name))
					} else {
						mod_row
					};
					let progress_col = progress_col.push(mod_row);

					let progress_col = progress_col.push(
						progress_bar(
							0.0..=progress.entries.len() as f32,
							(progress.entry_step + 1) as f32,
						)
						.height(Length::Fixed(10.0)),
					);

					let entry_row = Row::new().push(
						text(format!(
							"Entries: {} / {}",
							progress.entry_step + 1,
							progress.entries.len(),
						))
						.width(Length::Fill),
					);
					let entry_name = progress.entries.get(progress.entry_step);
					let entry_row = if let Some(name) = entry_name {
						entry_row.push(text(name))
					} else {
						entry_row
					};
					let progress_col = progress_col.push(entry_row);

					Some(progress_col)
				} else {
					None
				},
			)
			.into()
	}

	pub fn theme(_state: &Emtk) -> Theme {
		Theme::CatppuccinFrappe
	}

	pub fn subscription(&self) -> Subscription<Message> {
		event::listen().map(Message::IcedEvent)
	}
}

#[derive(Debug, Clone, Default)]
pub struct ProgressBar {
	entry_step: usize,
	entries: Vec<String>,
	mod_step: usize,
	mods: Vec<String>,
	rpk_step: usize,
	rpks: Vec<String>,
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
	Loading(ProgressBar),
	Loaded,
}

#[derive(Debug, Clone)]
pub enum Event {
	ExanimaLaunched(GameStartState),
	ProgressUpdated(ProgressBar),
}

fn load_mods(settings: AppSettings) -> impl Stream<Item = Event> {
	iced::stream::channel(0, |mut tx: Sender<Event>| async move {
		let mut progress_bar = ProgressBar::default();

		let exanima_exe = PathBuf::from(
			settings
				.exanima_exe
				.expect("error while getting exanima exe path"),
		);
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

		progress_bar.rpks = exanima_rpks
			.iter()
			.map(|path| {
				path.file_name()
					.expect("error while reading file name")
					.to_str()
					.expect("error while getting file name")
					.to_string()
			})
			.collect();
		progress_bar.mods = settings.mod_load_order.clone();

		for (i, path) in exanima_rpks.iter().enumerate() {
			let file_name = path
				.file_name()
				.expect("error while reading file name")
				.to_str()
				.expect("error while getting file name");

			let mut buf_reader =
				io::BufReader::new(fs::File::open(path).expect("error while opening exanima file"));
			let mut reader = Reader::new(&mut buf_reader);

			let mut exanima_format = Format::from_reader_with_ctx(&mut reader, ())
				.expect("error while reading exanima format");

			if let Format::Rpk(exanima_rpk) = &mut exanima_format {
				let mut exanima_sorted_entries = exanima_rpk.entries.to_vec();
				exanima_sorted_entries.sort_by(|a, b| a.offset.cmp(&b.offset));

				// TODO: design how mods should be considered enabled/disabled and how the mod load
				// order should be like
				// let enabled_plugins = settings.mods.iter().filter(|&plugin| {
				// 	plugin.info.config.plugin.id
				// });
				//
				// settings.mods;
				// mod_load_order is a vec of mod ids where the order matters that includes all mods in settings.mods
				// settings.mod_load_order;
				// enabled_mods will be a vec of mod ids where the order doesn't matter that will be used to filter mod_load_order
				// settings.enabled_mods;
				// FIX: currently will loop through all mods regardless if it's enabled/disabled
				// TODO: has_assets from config.toml should be used somewhere
				for (j, plugin) in settings
					.mods
					.iter()
					.filter(|&m| settings.mod_load_order.contains(&m.info.config.plugin.id))
					.enumerate()
				{
					let mod_path = PathBuf::from(&plugin.info.path)
						.join("assets")
						.join(file_name);
					if mod_path.exists() {
						let mut buf_reader = io::BufReader::new(
							fs::File::open(mod_path).expect("error while opening mod file"),
						);
						let mut reader = Reader::new(&mut buf_reader);
						let mod_format = Format::from_reader_with_ctx(&mut reader, ())
							.expect("error while reading mod format");

						if let Format::Rpk(mod_rpk) = mod_format {
							let mut sorted_mod_entries = mod_rpk.entries.to_vec();
							sorted_mod_entries.sort_by(|a, b| a.offset.cmp(&b.offset));

							progress_bar.entries = sorted_mod_entries
								.iter()
								.map(|entry| entry.name.clone())
								.collect();

							for (mod_entry_idx, mod_entry) in sorted_mod_entries.iter().enumerate()
							{
								if let Some(exanima_entry_idx) = exanima_sorted_entries
									.iter()
									.position(|e| e.name == mod_entry.name)
								{
									let mod_data = mod_rpk
										.data
										.get(mod_entry_idx)
										.expect("error while getting mod rpk data");
									let rpk_data = exanima_rpk
										.data
										.get_mut(exanima_entry_idx)
										.expect("error while getting exanima rpk data");
									*rpk_data = mod_data.clone();
								} else {
									// TODO: Verify this works
									// add the mod's entry to exanima's rpk file
									exanima_sorted_entries.push(mod_entry.clone());
									exanima_rpk.data.push(mod_rpk.data[mod_entry_idx].clone());
								}
								tx.send(Event::ExanimaLaunched(GameStartState::Loading(
									progress_bar.clone(),
								)))
								.await
								.expect("error while sending progress of entry to channel");
								progress_bar.entry_step = mod_entry_idx;
							}
						}
					}
					tx.send(Event::ExanimaLaunched(GameStartState::Loading(
						progress_bar.clone(),
					)))
					.await
					.expect("error while sending progress of mod to channel");
					progress_bar.mod_step = j;
				}
				let mut prev_offset = 0;
				let mut prev_size = 0;
				for (i, exanima_data) in exanima_rpk.data.iter().enumerate() {
					let entry = exanima_sorted_entries
						.get_mut(i)
						.expect("error while getting exanima rpk entry");
					entry.offset = prev_offset + prev_size;
					entry.size = exanima_data.len() as u32;
					prev_offset = entry.offset;
					prev_size = entry.size;
				}
				exanima_sorted_entries.sort_by(|a, b| a.name.cmp(&b.name));
				exanima_rpk.entries = exanima_sorted_entries;
			};

			// let cache_path = get_local_dir().join("AssetCache").join(file_name);
			// if !cache_path.exists() {
			// 	fs::create_dir_all(
			// 		cache_path
			// 			.parent()
			// 			.expect("error while getting parent of cache path"),
			// 	)
			// 	.expect("error while creating cache directory");
			// }
			// let mut cache_buf_writer = io::BufWriter::new(
			// 	fs::File::create(cache_path).expect("error while creating cache file"),
			// );
			// let mut cache_writer = Writer::new(&mut cache_buf_writer);
			// exanima_format
			// 	.to_writer(&mut cache_writer, ())
			// 	.expect("error while serializing to cache file");

			progress_bar.rpk_step = i;
			tx.send(Event::ExanimaLaunched(GameStartState::Loading(
				progress_bar.clone(),
			)))
			.await
			.expect("error while sending progress to channel");
		}

		tx.send(Event::ExanimaLaunched(GameStartState::Loaded))
			.await
			.expect("error while sending finished state to channel");
	})
}

async fn get_latest_release() -> anyhow::Result<Release> {
	let url = "https://codeberg.org/api/v1/repos/ExanimaModding/Toolkit/releases/latest";

	let release: Release = ureq::get(url).call()?.into_json()?;

	Ok(release)
}
