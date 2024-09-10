use std::{fs, path::PathBuf};

use iced::{
	widget::{button, checkbox, container, text, Column, Row},
	Alignment, Element, Length, Task,
};

#[derive(Debug, Clone)]
pub enum Action {
	DeveloperToggled(bool),
	ExplainToggled(bool),
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

		let row_padding = 5;
		container(
			Column::new()
				.push(
					Row::new()
						.push(button("Clear Cache").on_press(Message::CacheCleared))
						.push(
							container(text(format!(
								"Size: {:.2} GBs",
								self.cache_size as f32 / 1_000_000_000.
							)))
							.padding(5),
						)
						.padding(row_padding)
						.align_y(Alignment::Center),
				)
				.push(
					Row::new()
						.push(
							checkbox("Developer Mode", self.developer_enabled)
								.on_toggle(Message::DeveloperToggled),
						)
						.padding(row_padding),
				)
				.push_maybe(if self.developer_enabled {
					Some(
						Row::new()
							.push(
								checkbox("Explain UI Layout", self.explain_enabled)
									.on_toggle(Message::ExplainToggled),
							)
							.padding(row_padding),
					)
				} else {
					None
				}),
		)
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}
}

// TODO: move cache_path into a more appropriate file
pub fn cache_path(exanima_exe: String) -> PathBuf {
	PathBuf::from(exanima_exe)
		.parent()
		.unwrap()
		.join(".emtk")
		.join("cache")
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
