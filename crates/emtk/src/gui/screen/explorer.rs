use std::{
	fs,
	io::{self, Write},
	path::PathBuf,
	sync::Arc,
};

use exparser::{deku::prelude::*, rpk::Rpk, Format};
use human_bytes::human_bytes;
use iced::{
	widget::{
		button, container, horizontal_rule, horizontal_space, scrollable, svg, text, text_input,
		Column, Row,
	},
	Alignment, Color, Element, Length, Task,
};
use nucleo::{
	pattern::{CaseMatching, Normalization},
	Nucleo,
};
use rfd::FileDialog;

use crate::gui::{
	constants::{ARROW_LEFT, FOLDER, SQUARE_ARROW_OUT},
	theme,
};

#[derive(Debug, Default, Clone)]
pub struct Metadata {
	name: String,
	size: u64,
}

impl Metadata {
	pub fn new(name: String, size: u64) -> Self {
		Self { name, size }
	}
}

pub struct Explorer {
	matcher: Nucleo<usize>,
	metadata: Metadata,
	rpk: Option<Rpk>,
	rpk_paths: Vec<PathBuf>,
	query: String,
}

#[derive(Debug, Clone)]
pub enum Message {
	EntryExported((usize, String)),
	EntryImported,
	EntryRestored,
	Queried(String),
	RpkDialog,
	RpkSelected(Option<PathBuf>),
}

impl Explorer {
	pub fn new(rpk_paths: Vec<PathBuf>) -> Self {
		Self {
			rpk_paths,
			..Default::default()
		}
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::EntryExported((index, entry_name)) => {
				if let Some(rpk) = &mut self.rpk {
					let entry_data = rpk.data.get(index).unwrap();
					// TODO: add_filter and detect file type of entry
					if let Some(path) = FileDialog::new().set_file_name(entry_name).save_file() {
						let mut writer = io::BufWriter::new(fs::File::create_new(path).unwrap());
						writer.write_all(entry_data.as_slice()).unwrap();
					}
				}
			}
			// TODO: implementing importing means modifying the vanilla rpk files
			Message::EntryImported => (),
			Message::EntryRestored => (),
			Message::Queried(query) => {
				self.matcher.pattern.reparse(
					0,
					query.as_str(),
					CaseMatching::Ignore,
					Normalization::Smart,
					false,
				);
				self.matcher.tick(10);
				self.query = query;
			}
			Message::RpkDialog => {
				if let Some(path) = FileDialog::new()
					.add_filter("Rayform Package", &["rpk"])
					.pick_file()
				{
					return Task::done(Message::RpkSelected(Some(path)));
				}
			}
			Message::RpkSelected(path) => match path {
				Some(path) => {
					let file = fs::File::open(&path).unwrap();
					let mut buf_reader = io::BufReader::new(&file);
					let mut reader = Reader::new(&mut buf_reader);
					let format = Format::from_reader_with_ctx(&mut reader, ()).unwrap();
					if let Format::Rpk(rpk) = format {
						self.matcher =
							Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1);
						rpk.entries.iter().enumerate().for_each(|(index, entry)| {
							self.matcher.injector().push(index, |_index, haystack| {
								if let Some(haystack) = haystack.first_mut() {
									*haystack = entry.name.as_str().into();
								}
							});
						});
						self.metadata = Metadata::new(
							path.file_name().unwrap().to_str().unwrap().to_string(),
							file.metadata().unwrap().len(),
						);
						self.rpk = Some(rpk);
					};
				}
				None => {
					self.metadata = Metadata::default();
					self.rpk = None;
				}
			},
		};

		Task::none()
	}

	pub fn view(&self) -> Element<Message> {
		container(if let Some(rpk) = &self.rpk {
			self.view_entries(rpk)
		} else {
			self.view_packages()
		})
		.padding(12)
		.into()
	}

	fn view_entries(&self, rpk: &Rpk) -> Element<Message> {
		let search_results = self
			.matcher
			.snapshot()
			.matched_items(..)
			.map(|item| &rpk.entries[item.data.to_owned()])
			.collect::<Vec<_>>();
		let search_results = if search_results.is_empty() {
			rpk.entries.iter().collect()
		} else {
			search_results
		};

		let spacing = 6;
		Column::new()
			.push(
				Column::new()
					.push(
						Row::new()
							.push(text(&self.metadata.name).size(36))
							.push(horizontal_space())
							.push(text(human_bytes(self.metadata.size as f64)).size(36)),
					)
					.push(horizontal_rule(1))
					.spacing(spacing),
			)
			.push(
				Row::new()
					.push(
						button(
							svg(svg::Handle::from_memory(ARROW_LEFT))
								.width(Length::Shrink)
								.height(Length::Fixed(16.))
								.style(theme::svg),
						)
						.padding(6)
						.height(Length::Fixed(31.))
						.on_press(Message::RpkSelected(None))
						.style(theme::transparent_button),
					)
					.push(
						text_input("Search by entry name...", self.query.as_str())
							.on_input(Message::Queried),
					),
			)
			.push(
				scrollable(
					// PERF: slow with thousands of results, use infinite list widget
					Column::with_children(search_results.iter().enumerate().map(
						|(index, entry)| {
							Row::new()
								.push(text(index + 1).width(Length::Fixed(38.)))
								.push(text(entry.name.to_owned()))
								.push(horizontal_space())
								.push(text(human_bytes(entry.size)))
								.push(
									button(text("Restore")).style(button::danger),
									// .on_press(Message::EntryRestored),
								)
								.push(
									button(
										Row::new()
											.push(text("Import"))
											.push(
												container(
													svg(svg::Handle::from_memory(SQUARE_ARROW_OUT))
														.width(Length::Shrink)
														.height(Length::Fixed(16.))
														.style(theme::svg_button),
												)
												.height(Length::Fixed(21.))
												.align_y(Alignment::Center),
											)
											.spacing(2),
									)
									.style(button::danger),
									// .on_press(Message::EntryImported),
								)
								.push(
									button(
										Row::new()
											.push(text("Export"))
											.push(
												container(
													svg(svg::Handle::from_memory(SQUARE_ARROW_OUT))
														.width(Length::Shrink)
														.height(Length::Fixed(16.))
														.style(theme::svg_button),
												)
												.height(Length::Fixed(21.))
												.align_y(Alignment::Center),
											)
											.spacing(2),
									)
									.on_press(Message::EntryExported((
										index,
										entry.name.to_owned(),
									))),
								)
								.align_y(Alignment::Center)
								.spacing(6)
								.into()
						},
					))
					.spacing(1),
				)
				.spacing(spacing),
			)
			.spacing(spacing)
			.into()
	}

	fn view_packages(&self) -> Element<Message> {
		let spacing = 6;
		Column::new()
			.push(
				Column::with_children(self.rpk_paths.iter().map(|path| {
					let file_size =
						human_bytes(fs::File::open(path).unwrap().metadata().unwrap().len() as f64);
					button(
						Row::new()
							.push(
								svg(svg::Handle::from_memory(FOLDER))
									.width(Length::Shrink)
									.height(Length::Fixed(20.))
									.style(theme::svg),
							)
							.push(text(path.file_name().unwrap().to_str().unwrap()))
							.push(horizontal_space())
							.push(text(file_size))
							.spacing(12),
					)
					.on_press(Message::RpkSelected(Some(path.to_owned())))
					.style(theme::transparent_button)
					.into()
				}))
				.spacing(1),
			)
			.push(
				container(
					button(
						Row::new()
							.push(text("Load a Package"))
							.push(
								container(
									svg(svg::Handle::from_memory(SQUARE_ARROW_OUT))
										.width(Length::Shrink)
										.height(Length::Fixed(16.))
										.style(theme::svg_button),
								)
								.height(Length::Fixed(21.))
								.align_y(Alignment::Center),
							)
							.spacing(2),
					)
					.on_press(Message::RpkDialog)
					.style(button::primary),
				)
				.width(Length::Fill)
				.align_x(Alignment::Center),
			)
			.spacing(spacing)
			.into()
	}
}

impl Default for Explorer {
	fn default() -> Self {
		Self {
			matcher: Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1),
			metadata: Metadata::default(),
			rpk: Option::default(),
			rpk_paths: Vec::default(),
			query: String::default(),
		}
	}
}
