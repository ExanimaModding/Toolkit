use std::{fs, path::PathBuf};

use human_bytes::human_bytes;
use iced::{
	widget::{button, checkbox, container, horizontal_rule, scrollable, svg, text, Column, Row},
	Alignment, Color, Element, Length, Task,
};

use crate::gui::constants::SQUARE_ARROW_OUT;

#[derive(Debug, Clone)]
pub enum Action {
	DeveloperToggled(bool),
	ExplainToggled(bool),
	ViewChangelog,
	None,
}

#[derive(Debug, Default, Clone)]
pub struct Settings {
	cache_size: u64,
	developer_enabled: bool,
	explain_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
	CacheChecked,
	CacheCleared,
	CacheSize(u64),
	CacheOpened,
	Changelog,
	DeveloperToggled(bool),
	ExplainToggled(bool),
}

impl Settings {
	pub fn new(developer_enabled: bool, explain_enabled: bool) -> (Self, Task<Message>) {
		(
			Self {
				developer_enabled,
				explain_enabled,
				..Default::default()
			},
			Task::done(Message::CacheChecked),
		)
	}

	pub fn update(
		&mut self,
		message: Message,
		app_state: &mut crate::gui::state::AppState,
	) -> (Task<Message>, Action) {
		match message {
			Message::CacheChecked => {
				return (
					Task::perform(
						cache_size(cache_path(app_state.settings.exanima_exe.clone().unwrap())),
						Message::CacheSize,
					),
					Action::None,
				)
			}
			Message::CacheCleared => {
				return (
					Task::perform(
						clear_cache(cache_path(app_state.settings.exanima_exe.clone().unwrap())),
						|_| Message::CacheChecked,
					),
					Action::None,
				)
			}
			Message::CacheSize(cache_size) => self.cache_size = cache_size,
			Message::CacheOpened => {
				open::that(cache_path(app_state.settings.exanima_exe.clone().unwrap())).unwrap()
			}
			Message::Changelog => return (Task::none(), Action::ViewChangelog),
			Message::DeveloperToggled(developer_enabled) => {
				self.developer_enabled = developer_enabled;
				return (Task::none(), Action::DeveloperToggled(developer_enabled));
			}
			Message::ExplainToggled(explain_enabled) => {
				self.explain_enabled = explain_enabled;
				return (Task::none(), Action::ExplainToggled(explain_enabled));
			}
		};

		(Task::none(), Action::None)
	}

	pub fn view(&self) -> Element<Message> {
		// let col = Column::new().push(self.version());

		let spacing = 6;
		let category_size = 24;
		scrollable(
			container(
				Column::new()
					.push(
						Column::new()
							.push(text("Settings").size(36))
							.push(horizontal_rule(1))
							.spacing(spacing),
					)
					.push(
						Column::new()
							.push(text("About").size(category_size))
							.push(button("View Changelog").on_press(Message::Changelog))
							.push(horizontal_rule(1))
							.spacing(spacing),
					)
					.push(
						Column::new()
							.push(text("Cache").size(category_size))
							.push(
								Row::new()
									.push(button("Clear Cache").on_press(Message::CacheCleared))
									.push(
										container(text(format!(
											"Size: {}",
											human_bytes(self.cache_size as f64)
										)))
										.padding(5),
									)
									.align_y(Alignment::Center),
							)
							.push(
								button(
									Row::new()
										.push(text("Open Cache"))
										.push(
											container(
												svg(svg::Handle::from_memory(SQUARE_ARROW_OUT))
													.width(Length::Fixed(16.))
													.height(Length::Fixed(16.))
													.style(|_theme, _status| svg::Style {
														color: Some(Color::BLACK),
													}),
											)
											.height(Length::Fixed(21.))
											.align_y(Alignment::Center),
										)
										.spacing(2),
								)
								.on_press(Message::CacheOpened),
							)
							.push(horizontal_rule(1))
							.spacing(spacing),
					)
					.push(
						Column::new()
							.push(text("Developer").size(category_size))
							.push(
								Row::new().push(
									checkbox("Developer Mode", self.developer_enabled)
										.on_toggle(Message::DeveloperToggled),
								),
							)
							.push_maybe(if self.developer_enabled {
								Some(
									Row::new().push(
										checkbox("Explain UI Layout", self.explain_enabled)
											.on_toggle(Message::ExplainToggled),
									),
								)
							} else {
								None
							})
							.push(horizontal_rule(1))
							.spacing(spacing),
					)
					.spacing(12),
			)
			.padding(12),
		)
		.into()
	}
}

// TODO: move cache_path into a more appropriate file
pub fn cache_path(exanima_exe: String) -> PathBuf {
	let path = PathBuf::from(exanima_exe)
		.parent()
		.unwrap()
		.join(".emtk")
		.join("cache");

	if !path.is_dir() {
		fs::create_dir_all(&path).unwrap();
	}

	path
}

pub async fn cache_size(cache_path: PathBuf) -> u64 {
	if !cache_path.is_dir() {
		return 0;
	}

	let mut total_size = 0;

	for entry in cache_path.read_dir().unwrap().flatten() {
		total_size += entry.metadata().unwrap().len();
	}

	total_size
}

pub async fn clear_cache(cache_path: PathBuf) {
	if !cache_path.is_dir() {
		return;
	}

	fs::remove_dir_all(cache_path).unwrap();
}
