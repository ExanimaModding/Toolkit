use std::{fs, io, path::PathBuf, time::Instant};

use exparser::{deku::prelude::*, Format};
use iced::{
	futures::{channel::mpsc::Sender, SinkExt, Stream, StreamExt},
	stream, task,
	widget::{button, container, progress_bar, text, Column, Row},
	Alignment, Border, Color, Element, Length, Padding, Size, Task, Theme,
};
use lilt::{Animated, Easing};
use tokio::time::Duration;

use crate::{
	config::Config,
	gui::{constants::FADE_DURATION, missing_mods, path_by_id},
};

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
	fade: Animated<bool, Instant>,
	progress_completion: Animated<bool, Instant>,
	progress_increment: Animated<f32, Instant>,
	size: Option<Size>,
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
	pub fn new(config: Config, size: Size) -> (Self, Task<Message>) {
		let now = Instant::now();

		let (task, handle) = Task::stream(load_mods(config).map(Message::Event)).abortable();
		(
			Self {
				bar: Bar::default(),
				fade: Animated::new(false)
					.duration(FADE_DURATION as f32)
					.easing(Easing::EaseOut)
					.delay(0.)
					.auto_start(true, now),
				progress_completion: Animated::new(false)
					.duration(500.)
					.easing(Easing::Linear)
					.delay(0.),
				progress_increment: Animated::new(0.0).easing(Easing::Linear).duration(500.),
				size: Some(size),
				handle,
			},
			task,
		)
	}

	pub fn update(&mut self, message: Message) -> Action {
		let now = Instant::now();

		match message {
			Message::Canceled => {
				self.handle.abort();
				self.fade.transition(false, now);
				return Action::Canceled;
			}
			Message::Event(event) => match event {
				Event::ProgressCompleted(bar) => {
					self.bar = bar;
					self.progress_completion.transition(true, now);
					return Action::ExanimaLaunched;
				}
				Event::ProgressUpdated(bar) => {
					self.bar = bar;
					self.progress_increment
						.transition(self.bar.current_step as f32, now);
				}
			},
			Message::SizeChanged(size) => self.size = Some(size),
			Message::Tick => (),
		};

		Action::None
	}

	// TODO: add logs with tracing crate
	pub fn view(&self) -> Element<Message> {
		let now = Instant::now();

		let content = container(
			Column::new()
				.push(if self.bar.steps.is_empty() {
					Column::new().push(
						container(
							text("Loading...")
								.width(Length::Fill)
								.align_x(Alignment::Center),
						)
						.padding(24.),
					)
				} else {
					Column::new()
						.push(
							container(Row::new().push(text(self.bar.title.clone())))
								.padding(Padding::new(0.).bottom(12)),
						)
						.push(container(
							Column::new()
								.push(
									Row::new()
										.push(
											text(format!(
												"{} / {} Packages",
												self.bar.current_step,
												self.bar.steps.len()
											))
											.width(Length::Fill),
										)
										.push_maybe(
											self.bar.steps.get(self.bar.current_step).map(text),
										),
								)
								.push(
									progress_bar(
										0.0..=self.bar.steps.len() as f32,
										self.progress_increment.animate(|step| step, now),
									)
									.height(Length::Fixed(16.))
									.style(move |theme: &Theme| {
										let palette = theme.extended_palette();
										let animate_alpha = self.fade.animate_bool(0., 1., now);

										let mut style = progress_bar::primary(theme);
										style.background =
											style.background.scale_alpha(animate_alpha);
										style.bar = {
											let primary = palette.primary.strong.color;
											let success = palette.success.strong.color;
											Color::from_rgba(
												self.progress_completion
													.animate_bool(primary.r, success.r, now),
												self.progress_completion
													.animate_bool(primary.g, success.g, now),
												self.progress_completion
													.animate_bool(primary.b, success.b, now),
												animate_alpha,
											)
											.into()
										};
										style.border = Border::default().rounded(8);
										style
									}),
								),
						))
				})
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
						.on_press(Message::Canceled)
						.style(move |theme: &Theme, status| {
							let palette = theme.extended_palette();
							let animate_alpha = self.fade.animate_bool(0., 1., now);

							let mut style = button::primary(theme, status);
							style.background = Some(match status {
								button::Status::Hovered => {
									let primary = palette.primary.weak.color;
									let success = palette.success.weak.color;
									Color::from_rgba(
										self.progress_completion
											.animate_bool(primary.r, success.r, now),
										self.progress_completion
											.animate_bool(primary.g, success.g, now),
										self.progress_completion
											.animate_bool(primary.b, success.b, now),
										animate_alpha,
									)
									.into()
								}
								_ => {
									let primary = palette.primary.strong.color;
									let success = palette.success.strong.color;
									Color::from_rgba(
										self.progress_completion
											.animate_bool(primary.r, success.r, now),
										self.progress_completion
											.animate_bool(primary.g, success.g, now),
										self.progress_completion
											.animate_bool(primary.b, success.b, now),
										animate_alpha,
									)
									.into()
								}
							});
							style.text_color = style.text_color.scale_alpha(animate_alpha);
							style
						}),
					)
					.padding(Padding::new(0.).top(12))
					.width(Length::Fill)
					.align_x(Alignment::Center),
				),
		)
		.padding(12)
		.style(move |theme| {
			let palette = theme.palette();
			let animate_alpha = self.fade.animate_bool(0., 1., now);

			container::Style::default()
				.color(palette.text.scale_alpha(animate_alpha))
				.background(palette.background.scale_alpha(animate_alpha))
				.border(Border::default().rounded(8))
		});

		if let Some(size) = self.size {
			content.width(size.width).into()
		} else {
			content.into()
		}
	}
}

fn load_mods(config: Config) -> impl Stream<Item = Event> {
	stream::channel(0, |mut tx: Sender<Event>| async move {
		tokio::time::sleep(Duration::from_millis(FADE_DURATION)).await;
		let mut bar = Bar::default();

		let exanima_exe = PathBuf::from(
			config
				.exanima_exe
				.expect("error while getting exanima exe path"),
		);

		let missing_mods = missing_mods(&config.load_order, &exanima_exe);

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

				for (mod_id, enabled) in &config.load_order {
					if !enabled || missing_mods.contains(&(mod_id.clone(), *enabled)) {
						continue;
					}

					let mod_path = if let Some(mod_path) = path_by_id(&exanima_exe, mod_id) {
						mod_path
					} else {
						continue;
					}
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

			let emtk_data_path = exanima_exe.parent().unwrap().join(".emtk");
			// TODO: replace cache_path variable to use a cache_path function
			let cache_path = emtk_data_path.join("cache").join(file_name);
			if !cache_path.is_dir() {
				fs::create_dir_all(
					cache_path
						.parent()
						.expect("error while getting parent of cache path"),
				)
				.expect("error while creating cache directory");
			}
			let mut cache_buf_writer = io::BufWriter::new(
				fs::File::create(cache_path).expect("error while creating cache file"),
			);
			let mut cache_writer = Writer::new(&mut cache_buf_writer);
			exanima_format
				.to_writer(&mut cache_writer, ())
				.expect("error while serializing to cache file");

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
