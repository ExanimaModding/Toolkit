use std::{
	collections::HashMap,
	fs,
	io::{self, Write},
	path::PathBuf,
	sync::Arc,
	time::Instant,
};

use exparser::{
	deku::prelude::*,
	rpk::{self, Rpk},
	Context, Format,
};
use human_bytes::human_bytes;
use iced::{
	widget::{
		button, container, horizontal_rule, horizontal_space, mouse_area, scrollable, svg, text,
		text_input, Column, Row,
	},
	window, Alignment, Element, Length, Subscription, Task,
};
use lilt::{Animated, Easing};
use nucleo::{
	pattern::{CaseMatching, Normalization},
	Nucleo,
};
use rfd::FileDialog;

use crate::gui::{
	constants::FADE_DURATION,
	theme,
	widget::{
		list::{self, List},
		tooltip,
	},
	Icon,
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
	fade: Animated<bool, Instant>,
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
	Tick,
	TooltipHide,
	TooltipShow,
}

impl Explorer {
	pub fn new(rpk_paths: Vec<PathBuf>) -> Self {
		Self {
			rpk_paths,
			..Default::default()
		}
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		let now = Instant::now();

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
					// BUG: saving to file that already exists freezes app until a windows dialog appears
					if let Some(path) = FileDialog::new().set_file_name(entry.name).save_file() {
						let mut writer = io::BufWriter::new(fs::File::create(path).unwrap());
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
			// TODO: support multiple selections
			Message::RpkDialog => {
				if let Some(path) = FileDialog::new()
					.add_filter("Rayform Package", &["rpk"])
					.pick_file()
				{
					return Task::done(Message::RpkSelected(Some(path)));
				}
			}
			Message::RpkSelected(path) => {
				self.query = String::new();
				match path {
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
				}
			}
			Message::Tick => (),
			Message::TooltipHide => {
				if !self.fade.in_progress(now) {
					self.fade.transition_instantaneous(false, now);
				}
			}
			Message::TooltipShow => {
				if !self.fade.in_progress(now) {
					self.fade.transition(true, now);
				}
			}
		};

		Task::none()
	}

	pub fn view(&self, icons: &HashMap<Icon, svg::Handle>) -> Element<Message> {
		let now = Instant::now();
		let spacing = 6;
		let square_arrow_out_up_right = icons.get(&Icon::SquareArrowOutUpRight).unwrap().clone();

		container(if let Some(_rpk) = &self.rpk {
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
								svg(icons.get(&Icon::ArrowLeft).unwrap().clone())
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
						List::new(&self.content, move |index, entry| {
							Row::new()
								.push(text(index + 1).width(Length::Fixed(38.)))
								.push(text(entry.name.to_owned()))
								.push(horizontal_space())
								.push(text(human_bytes(entry.size)))
								.push(
									mouse_area(
										tooltip(
											button(text("Restore")).style(button::danger),
											// .on_press(Message::EntryRestored),
											"Restore to original",
										)
										.style(move |theme| {
											theme::tooltip(
												theme,
												self.fade.animate_bool(0., 1., now),
											)
										}),
									)
									.on_enter(Message::TooltipShow)
									.on_move(|_| Message::TooltipShow)
									.on_exit(Message::TooltipHide),
								)
								.push(
									mouse_area(
										tooltip(
											button(
												Row::new()
													.push(text("Import"))
													.push(
														container(
															svg(square_arrow_out_up_right.clone())
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
											"Replace with file",
										)
										.style(move |theme| {
											theme::tooltip(
												theme,
												self.fade.animate_bool(0., 1., now),
											)
										}),
									)
									.on_enter(Message::TooltipShow)
									.on_move(|_| Message::TooltipShow)
									.on_exit(Message::TooltipHide),
								)
								.push(
									mouse_area(
										tooltip(
											button(
												Row::new()
													.push(text("Export"))
													.push(
														container(
															svg(square_arrow_out_up_right.clone())
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
											"Save to file",
										)
										.padding(8)
										.style(move |theme| {
											theme::tooltip(
												theme,
												self.fade.animate_bool(0., 1., now),
											)
										}),
									)
									.on_enter(Message::TooltipShow)
									.on_move(|_| Message::TooltipShow)
									.on_exit(Message::TooltipHide),
								)
								.align_y(Alignment::Center)
								.spacing(6)
								.into()
						}),
					)
					.spacing(spacing),
				)
				.spacing(spacing)
		} else {
			Column::new()
				.push(
					Column::with_children(self.rpk_paths.iter().map(|path| {
						let file_size = human_bytes(
							fs::File::open(path).unwrap().metadata().unwrap().len() as f64,
						);
						button(
							Row::new()
								.push(
									svg(icons.get(&Icon::Folder).unwrap().clone())
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
						mouse_area(
							tooltip(
								button(
									Row::new()
										.push(text("Load a Package"))
										.push(
											container(
												svg(icons
													.get(&Icon::SquareArrowOutUpRight)
													.unwrap()
													.clone())
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
								"Custom Rayform Package",
							)
							.style(move |theme| {
								theme::tooltip(theme, self.fade.animate_bool(0., 1., now))
							}),
						)
						.on_enter(Message::TooltipShow)
						.on_move(|_| Message::TooltipShow)
						.on_exit(Message::TooltipHide),
					)
					.width(Length::Fill)
					.align_x(Alignment::Center),
				)
				.spacing(spacing)
		})
		.padding(12)
		.into()
	}

	pub fn subscription(&self) -> Subscription<Message> {
		let now = Instant::now();

		if self.fade.in_progress(now) {
			window::frames().map(|_| Message::Tick)
		} else {
			Subscription::none()
		}
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
			fade: Animated::new(false)
				.duration(FADE_DURATION as f32)
				.easing(Easing::EaseOut)
				.delay(500.),
		}
	}
}
