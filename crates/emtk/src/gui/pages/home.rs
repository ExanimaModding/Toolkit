use std::{
	fs, io,
	path::{Path, PathBuf},
};

use exparser::{deku::prelude::*, Format};
use tokio::sync::mpsc::Sender;

use crate::config::{get_local_dir, AppSettings};

use async_stream::stream;
use iced::{
	futures::Stream,
	widget::{
		checkbox::Checkbox,
		container::{self},
		pane_grid, progress_bar, Button, Column, Container, PaneGrid, Row, Rule, Text,
	},
	Element, Length, Task, Theme,
};

#[derive(Debug, Clone)]
pub enum Message {
	OpenUrl(String),
	StartGame(GameStartType),
	LoadSettings(crate::config::AppSettings),
	LaunchExanima(GameStartState),
	UpdateModOrder,
	UpdateModSettings(bool),
	UpdateProgress(ProgressBar),
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
	Error(String),
}

#[derive(Debug, Clone)]
pub struct Home {
	plugin_panes: pane_grid::State<Plugin>,
	game_start_state: GameStartState,
	settings: AppSettings,
}

impl Default for Home {
	fn default() -> Self {
		let (plugin_panes, _) = pane_grid::State::new(Plugin::default());
		Self {
			plugin_panes,
			game_start_state: GameStartState::default(),
			settings: AppSettings::default(),
		}
	}
}

impl Home {
	pub fn view(&self) -> Element<Message> {
		Container::new(
			Column::new()
				.push(Text::new("Welcome to the Exanima Modding Toolkit Launcher!").size(20))
				.push(Rule::horizontal(1))
				.push(self.mods_list())
				.push(self.show_home_section())
				.spacing(10),
		)
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}

	fn mods_list(&self) -> Element<Message> {
		Container::new(Text::new("WIP"))
			.width(Length::Fill)
			.height(Length::Fill)
			.into()
	}

	fn show_home_section(&self) -> Element<Message> {
		let col = Column::new();

		let col = col.push_maybe(
			if let GameStartState::Loading(progress) = &self.game_start_state {
				let progress_col = Column::new().push(
					progress_bar(
						0.0..=progress.rpks.len() as f32,
						(progress.rpk_step + 1) as f32,
					)
					.height(Length::Fixed(10.0)),
				);

				let rpk_row = Row::new().push(
					Text::new(format!(
						"Rpks: {} / {}",
						progress.rpk_step + 1,
						progress.rpks.len(),
					))
					.width(Length::Fill),
				);
				let rpk_name = progress.rpks.get(progress.rpk_step);
				let rpk_row = if let Some(name) = rpk_name {
					rpk_row.push(Text::new(name))
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
					Text::new(format!(
						"Mods: {} / {}",
						progress.mod_step + 1,
						progress.mods.len(),
					))
					.width(Length::Fill),
				);
				let mod_name = progress.mods.get(progress.mod_step);
				let mod_row = if let Some(name) = mod_name {
					mod_row.push(Text::new(name))
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
					Text::new(format!(
						"Entries: {} / {}",
						progress.entry_step + 1,
						progress.entries.len(),
					))
					.width(Length::Fill),
				);
				let entry_name = progress.entries.get(progress.entry_step);
				let entry_row = if let Some(name) = entry_name {
					entry_row.push(Text::new(name))
				} else {
					entry_row
				};
				let progress_col = progress_col.push(entry_row);

				Some(progress_col)
			} else {
				None
			},
		);

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
				self.game_start_state = GameStartState::Loading(ProgressBar::default());
				log::info!("Starting modded Exanima...");
				Task::stream(load_mods(self.settings.clone()))
			}
			Message::StartGame(GameStartType::Vanilla) => {
				// TODO: start vanilla exanima
				self.game_start_state = GameStartState::Loading(ProgressBar::default());
				log::info!("Starting vanilla Exanima...");
				Task::none()
			}
			Message::LoadSettings(settings) => {
				self.settings = settings;

				let (mut plugin_panes, pane) = pane_grid::State::new(Plugin::default());
				for (i, mod_setting) in self.settings.mods.iter().enumerate() {
					plugin_panes.split(
						pane_grid::Axis::Horizontal,
						pane,
						Plugin::new(
							i,
							None,
							mod_setting.info.config.plugin.enabled,
							mod_setting.info.config.plugin.name.clone(),
						),
					);
				}
				plugin_panes.close(pane);

				self.plugin_panes = plugin_panes;
				Task::none()
			}
			Message::LaunchExanima(state) => {
				self.game_start_state = state;
				// TODO: launch exanima
				// crate::launch_exanima();
				log::info!("Launching exanima...");
				Task::none()
			}
			Message::UpdateModOrder => Task::none(),
			Message::UpdateModSettings(mod_toggle) => {
				// TODO: save settings
				Task::none()
			}
			Message::UpdateProgress(progress) => {
				self.game_start_state = GameStartState::Loading(progress);
				Task::none()
			}
		};

		result.map(crate::gui::Message::HomePage)
	}
}

fn load_mods(settings: AppSettings) -> impl Stream<Item = Message> {
	stream! {
		let (tx, mut rx) = tokio::sync::mpsc::channel::<GameStartState>(1);

		tokio::spawn(async move {
			merge_mod_assets(tx, settings).await;
		});

		while let Some(state) = rx.recv().await {
			if let GameStartState::Loading(progress) = state {
				yield Message::UpdateProgress(progress)
			} else if let GameStartState::Loaded = state {
				yield Message::LaunchExanima(state)
			}
		}
	}
}

async fn merge_mod_assets(tx: Sender<GameStartState>, settings: AppSettings) {
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

						for (mod_entry_idx, mod_entry) in sorted_mod_entries.iter().enumerate() {
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
							tx.send(GameStartState::Loading(progress_bar.clone()))
								.await
								.expect("error while sending progress of entry to channel");
							progress_bar.entry_step = mod_entry_idx;
						}
					}
				}
				tx.send(GameStartState::Loading(progress_bar.clone()))
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
		tx.send(GameStartState::Loading(progress_bar.clone()))
			.await
			.expect("error while sending progress to channel");
	}

	tx.send(GameStartState::Loaded)
		.await
		.expect("error while sending finished state to channel");
}

#[derive(Debug, Default, Clone)]
pub struct Plugin {
	id: usize,
	order: Option<usize>,
	enabled: bool,
	name: String,
}

impl Plugin {
	fn new(id: usize, order: Option<usize>, enabled: bool, name: String) -> Self {
		Self {
			id,
			order,
			enabled,
			name,
		}
	}
}
