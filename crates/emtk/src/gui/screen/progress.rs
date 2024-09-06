use exparser::{deku::prelude::*, Format};
use lilt::{Animated, Easing};
use std::{fs, io, path::PathBuf, time::Instant};

use crate::config::{get_local_dir, AppSettings};
use iced::{
	futures::{channel::mpsc::Sender, SinkExt, Stream, StreamExt},
	task,
	widget::{button, container, progress_bar, text, Column, Row},
	window, Alignment, Background, Border, Element, Length, Padding, Size, Subscription, Task,
};

#[derive(Debug, Clone)]
pub enum Action {
	Canceled,
	ExanimaLaunched,
	None,
}

#[derive(Debug, Clone)]
pub enum Event {
	ProgressCompleted(Bar),
	ProgressUpdated(Bar),
}

#[derive(Debug, Clone, Default)]
pub struct Bar {
	pub current_step: usize,
	pub steps: Vec<String>,
	pub title: String,
}

#[derive(Debug, Clone)]
pub struct Progress {
	bar: Bar,
	size: Option<Size>,
	spinner_rotation: Animated<bool, Instant>,
	handle: task::Handle,
}

#[derive(Debug, Clone)]
pub enum Message {
	Canceled,
	Event(Event),
	SizeChanged(Size),
	Tick,
}

impl Progress {
	pub fn new(settings: AppSettings, size: Size) -> (Self, Task<Message>) {
		let now = Instant::now();

		let (task, handle) = Task::stream(load_mods(settings).map(Message::Event)).abortable();
		(
			Self {
				bar: Bar::default(),
				size: Some(size),
				spinner_rotation: Animated::new(false)
					.easing(Easing::Linear)
					.duration(900.)
					.repeat_forever()
					.auto_start(true, now),
				handle,
			},
			task,
		)
	}

	pub fn update(&mut self, message: Message) -> Action {
		match message {
			Message::Canceled => {
				self.handle.abort();
				Action::Canceled
			}
			Message::Event(event) => match event {
				Event::ProgressCompleted(bar) => {
					self.bar = bar;
					Action::ExanimaLaunched
				}
				Event::ProgressUpdated(bar) => {
					self.bar = bar;
					Action::None
				}
			},
			Message::SizeChanged(size) => {
				self.size = Some(size);
				Action::None
			}
			Message::Tick => Action::None,
		}
	}

	// TODO: add logs with tracing crate
	pub fn view(&self) -> Element<Message> {
		let now = Instant::now();

		let step_name = self.bar.steps.get(self.bar.current_step);
		let bar_header = Row::new().push(
			text(format!(
				"{} / {} Packages",
				self.bar.current_step,
				self.bar.steps.len()
			))
			.width(Length::Fill),
		);
		let bar_header = if let Some(name) = step_name {
			bar_header.push(text(name))
		} else {
			bar_header
		};

		let content = container(
			Column::new()
				.push(
					container(Row::new().push(text(self.bar.title.clone())))
						.padding(Padding::new(0.).bottom(12)),
				)
				.push(container(
					Column::new().push(bar_header).push(
						// TODO: add rounded corners
						progress_bar(
							0.0..=self.bar.steps.len() as f32,
							(self.bar.current_step + 1) as f32,
						)
						.height(Length::Fixed(16.))
						.style(|_theme| {
							let palette = &iced::theme::palette::EXTENDED_CATPPUCCIN_FRAPPE;
							iced::widget::progress_bar::Style {
								background: Background::Color(palette.background.weak.color),
								bar: Background::Color(palette.primary.strong.color),
								border: Border {
									radius: 8.0.into(),
									..Default::default()
								},
							}
						}),
					),
				))
				.push(
					container(
						button(
							if self.bar.current_step == self.bar.steps.len()
								&& self.bar.current_step != 0
							{
								"Close"
							} else {
								"Cancel"
							},
						)
						.on_press(Message::Canceled),
					)
					.padding(Padding::new(0.).top(12))
					.width(Length::Fill)
					.align_x(Alignment::Center),
				),
		)
		.padding(12)
		.style(|_theme| {
			let palette = iced::theme::Palette::CATPPUCCIN_FRAPPE;
			container::Style {
				text_color: Some(palette.text),
				background: Some(iced::Background::Color(palette.background)),
				border: iced::Border {
					radius: 8.0.into(),
					..Default::default()
				},
				..Default::default()
			}
		});

		if let Some(size) = self.size {
			content.width(size.width).into()
		} else {
			content.into()
		}
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let now = Instant::now();

		if self.spinner_rotation.in_progress(now) {
			window::frames().map(|_| Message::Tick)
		} else {
			Subscription::none()
		}
	}
}

fn load_mods(settings: AppSettings) -> impl Stream<Item = Event> {
	iced::stream::channel(0, |mut tx: Sender<Event>| async move {
		let mut bar = Bar::default();

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

		bar.steps = exanima_rpks
			.iter()
			.map(|path| {
				path.file_name()
					.expect("error while reading file name")
					.to_str()
					.expect("error while getting file name")
					.to_string()
			})
			.collect();

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
								bar.title = mod_entry.name.clone();
								tx.send(Event::ProgressUpdated(bar.clone()))
									.await
									.expect("error while sending progress of entry to channel");
							}
						}
					}
					// tx.send(Event::ProgressUpdated(bar.clone()))
					// 	.await
					// 	.expect("error while sending progress of mod to channel");
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

			// NOTE: code block writes to disk and is commented out for testing
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

			bar.current_step = i + 1;
			tx.send(Event::ProgressUpdated(bar.clone()))
				.await
				.expect("error while sending progress to channel");
		}

		bar.title = "Completed".to_string();
		tx.send(Event::ProgressCompleted(bar))
			.await
			.expect("error while sending finished state to channel");
	})
}
