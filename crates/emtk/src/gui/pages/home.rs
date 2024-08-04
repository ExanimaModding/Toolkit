use std::{fs, io, path::PathBuf};

use exparser::{deku::prelude::*, Format};

use super::changelog::GetLatestReleaseState;
use crate::{config::AppSettings, gui::constants};

use async_stream::stream;
use iced::{
	futures::Stream,
	widget::{
		self,
		markdown::{self},
		progress_bar, scrollable, Button, Column, Container, Row, Rule, Text,
	},
	Element, Length, Task,
};

#[derive(Debug, Clone)]
pub enum Message {
	OpenUrl(String),
	StartGame(GameStartType),
	LoadSettings(crate::config::AppSettings),
	LaunchGame,
	UpdateProgress(f32, String, f32, String),
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
	Loading(f32, String, f32, String),
	Loaded,
	Error(String),
}

#[derive(Debug, Default, Clone)]
pub struct Home {
	changelog: Vec<markdown::Item>,
	game_start_state: GameStartState,
	settings: AppSettings,
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
		let col = Column::new()
			.push(Container::new(Text::new("").height(Length::Fill)).height(Length::Fill));

		let col = if let GameStartState::Loading(entry_step, entry_name, rpk_step, rpk_name) =
			&self.game_start_state
		{
			let col = col.push(progress_bar(
				0.0..=self.settings.mod_load_order.len() as f32,
				*entry_step,
			));

			col.push(Text::new(format!(
				"{} / 12 {} {}",
				rpk_step, rpk_name, entry_name
			)))
		} else {
			col
		};

		let col = col.push(self.play_buttons());

		col.into()
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
				self.game_start_state =
					GameStartState::Loading(0.0, String::new(), 0.0, String::new());
				log::info!("Starting modded Exanima...");
				Task::stream(load_mods(
					self.settings.mod_load_order.clone(),
					self.settings
						.exanima_exe
						.clone()
						.expect("error while getting exanima_exe"),
				))
			}
			Message::StartGame(GameStartType::Vanilla) => {
				self.game_start_state =
					GameStartState::Loading(0.0, String::new(), 0.0, String::new());
				log::info!("Starting vanilla Exanima...");
				Task::none()
			}
			Message::LoadSettings(settings) => {
				self.settings = settings;
				Task::none()
			}
			Message::LaunchGame => {
				println!("Launching exanima");
				Task::none()
			}
			Message::UpdateProgress(entry_step, entry_name, rpk_step, rpk_name) => {
				self.game_start_state =
					GameStartState::Loading(entry_step, entry_name, rpk_step, rpk_name);
				Task::none()
			}
		};

		result.map(crate::gui::Message::HomePage)
	}
}

fn load_mods(mod_load_order: Vec<String>, exanima_exe: String) -> impl Stream<Item = Message> {
	// fn load_mods(mod_load_order: Vec<String>, exanima_exe: String) {
	stream! {
		let (tx, mut rx) = tokio::sync::mpsc::channel::<GameStartState>(1);

		tokio::spawn(async move {
			let exanima_exe_path = PathBuf::from(exanima_exe);
			let exanima_path = exanima_exe_path.parent().unwrap();
			let mut rpk_step = 0;
			// loop through each of the exanima's rpk files
			for entry in exanima_path
				.read_dir()
				.expect("error while reading exanima directory")
				.flatten()
			{
				let path = entry.path();
				let file_name = path
					.file_name()
					.expect("error while reading file name")
					.to_str()
					.expect("error while getting file name");
				if path.is_dir() || !file_name.ends_with(".rpk") {
					continue;
				}

				rpk_step += 1;
				let mut exanima_file = fs::File::open(&path).expect("error opening file");
				let mut buf_reader = io::BufReader::new(&mut exanima_file);
				let mut reader = Reader::new(&mut buf_reader);
				// exanima_format is the exanima's Textures.rpk
				let mut exanima_format =
					Format::from_reader_with_ctx(&mut reader, ()).expect("error reading format");

				// loop through each mod in the load order
				for (i, mod_name) in mod_load_order.iter().enumerate() {
					let mod_path = exanima_path
						.join("mods")
						.join(mod_name)
						.join("assets")
						.join(file_name);
					if !mod_path.exists() {
						continue;
					}
					// crate::loader::load_mod(mod_rpk_path, &mut format).expect("failed loading mod");
					let mut mod_file = fs::File::open(&mod_path).unwrap();
					let mut buf_reader = io::BufReader::new(&mut mod_file);
					let mut reader = Reader::new(&mut buf_reader);
					// mod_format is the mod's Textures.rpk
					let mod_format = Format::from_reader_with_ctx(&mut reader, ()).unwrap();
					if let Format::Rpk(mod_rpk) = mod_format {
						if let Format::Rpk(exanima_rpk) = &mut exanima_format {
							// loop through the mod's rpk file
							for (j, mod_entry) in mod_rpk.entries.iter().enumerate() {
								tx.send(GameStartState::Loading(
									j as f32,
									mod_entry.name.clone(),
									rpk_step as f32,
									file_name.to_string(),
								))
								.await
								.unwrap();

								// loop through exanima's rpk file
								for (k, exanima_entry) in exanima_rpk.entries.iter_mut().enumerate() {
									if mod_entry.name == exanima_entry.name {
										let mod_data =
											mod_rpk.data.get(j).expect("error getting mod rpk data");
										let rpk_data = exanima_rpk
											.data
											.get_mut(k)
											.expect("error getting exanima rpk data");
										*rpk_data = mod_data.clone();
									}
								}
							}
						}
					}
				}
				let mut prev_offset = 0;
				let mut prev_size = 0;
				if let Format::Rpk(exanima_rpk) = &mut exanima_format {
					let mut entries = exanima_rpk.entries.to_vec();
					entries.sort_by(|a, b| a.offset.cmp(&b.offset));
					for (i, exanima_data) in exanima_rpk.data.iter().enumerate() {
						let entry = entries
							.get_mut(i)
							.expect("error getting exanima entry");
						entry.offset = prev_offset + prev_size;
						entry.size = exanima_data.len() as u32;
						prev_offset = entry.offset;
						prev_size = entry.size;
					}
					exanima_rpk.entries = entries;
				}

				let cache_path = PathBuf::from("C:/Users/Dea/AppData/Local/Exanima Modding Toolkit")
					.join(file_name);
				let mut cache_file =
					fs::File::create(cache_path).expect("error while creating cache file");
				let mut cache_buf_writer = io::BufWriter::new(&mut cache_file);
				let mut cache_writer = Writer::new(&mut cache_buf_writer);
				exanima_format
					.to_writer(&mut cache_writer, ())
					.expect("error while serializing to cache file");
			}

			tx.send(GameStartState::Loaded).await.unwrap();
		});

		while let Some(state) = rx.recv().await {
			if let GameStartState::Loading(entry_step, entry_name, rpk_step, rpk_name) = state {
				yield Message::UpdateProgress(entry_step, entry_name, rpk_step, rpk_name)
			} else if let GameStartState::Loaded = state {
				yield Message::LaunchGame
			}
		}
	}
}
