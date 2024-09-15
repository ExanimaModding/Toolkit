use std::{
	fs,
	io::{self, Write},
	path::PathBuf,
	sync::Arc,
};

use exparser::{
	deku::prelude::*,
	rpk::{self, Rpk},
	Context, Format,
};
use human_bytes::human_bytes;
use iced::{
	widget::{
		button, container, horizontal_rule, horizontal_space, scrollable, svg, text, text_input,
		Column, Row,
	},
	Alignment, Element, Length, Task,
};
use nucleo::{
	pattern::{CaseMatching, Normalization},
	Nucleo,
};
use rfd::FileDialog;

use crate::gui::{
	constants::{ARROW_LEFT, FOLDER, SQUARE_ARROW_OUT},
	theme,
	widget::list::{self, List},
};

#[derive(Debug, Default, Clone)]
pub struct Metadata {
	name: String,
	path: PathBuf,
	size: u64,
}

impl Metadata {
	pub fn new(name: String, path: PathBuf, size: u64) -> Self {
		Self { name, path, size }
	}
}

pub struct Explorer {
	content: list::Content<rpk::Entry>,
	matcher: Nucleo<usize>,
	metadata: Metadata,
	rpk: Option<Rpk>,
	rpk_paths: Vec<PathBuf>,
	query: String,
}

#[derive(Debug, Clone)]
pub enum Message {
	EntryExported(rpk::Entry),
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
			Message::EntryExported(entry) => {
				let mut buf_reader =
					io::BufReader::new(fs::File::open(&self.metadata.path).unwrap());
				let mut reader = Reader::new(&mut buf_reader);
				let mut ctx = Context::default();
				ctx.rpk.entries = Some(vec![entry.to_owned()]);
				let format = Format::from_reader_with_ctx(&mut reader, ctx).unwrap();
				if let Format::Rpk(rpk) = format {
					// TODO: add_filter and detect file type of entry
					if let Some(path) = FileDialog::new().set_file_name(entry.name).save_file() {
						let mut writer = io::BufWriter::new(fs::File::create_new(path).unwrap());
						writer.write_all(rpk.data[0].as_slice()).unwrap();
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

				if let Some(rpk) = &self.rpk {
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
					self.content = list::Content::from_iter(
						// PERF: potentially thousands of entries are getting cloned here
						search_results.iter().map(|&entry| entry.to_owned()),
					)
				}
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
					let mut ctx = Context::default();
					ctx.rpk.entries_only = true;
					let format = Format::from_reader_with_ctx(&mut reader, ctx).unwrap();
					if let Format::Rpk(rpk) = format {
						// PERF: potentially thousands of entries are getting cloned here
						self.content = list::Content::from_iter(rpk.entries.to_owned());
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
							path,
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
		container(if let Some(_rpk) = &self.rpk {
			self.view_entries()
		} else {
			self.view_packages()
		})
		.padding(12)
		.into()
	}

	fn view_entries(&self) -> Element<Message> {
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
					// FIX: add spacing(1) to list but fix scrollbar appearing while loading
					List::new(&self.content, |index, entry| {
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
								.on_press(Message::EntryExported(entry.to_owned())),
							)
							.align_y(Alignment::Center)
							.spacing(6)
							.into()
					}),
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
			content: list::Content::default(),
			matcher: Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1),
			metadata: Metadata::default(),
			rpk: Option::default(),
			rpk_paths: Vec::default(),
			query: String::default(),
		}
	}
}
