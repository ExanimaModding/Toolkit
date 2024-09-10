use std::{fs, path::PathBuf};

use iced::{
	widget::{button, container, text, Column, Row},
	Alignment, Element, Length, Task,
};

#[derive(Debug, Default, Clone)]
pub struct Settings {
	cache_size: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
	CacheChecked,
	CacheCleared,
	CacheSize(u64),
}

impl Settings {
	pub fn new() -> (Self, Task<Message>) {
		(
			Self {
				..Default::default()
			},
			Task::done(Message::CacheChecked),
		)
	}

	pub fn update(
		&mut self,
		message: Message,
		app_state: &mut crate::gui::state::AppState,
	) -> Task<Message> {
		match message {
			Message::CacheChecked => Task::perform(
				cache_size(cache_path(app_state.settings.exanima_exe.clone().unwrap())),
				Message::CacheSize,
			),
			Message::CacheCleared => Task::perform(
				clear_cache(cache_path(app_state.settings.exanima_exe.clone().unwrap())),
				|_| Message::CacheChecked,
			),
			Message::CacheSize(cache_size) => {
				self.cache_size = cache_size;
				Task::none()
			}
		}
	}

	pub fn view(&self) -> Element<Message> {
		// let col = Column::new().push(self.version());

		container(
			Column::new().push(container(
				Row::new()
					.push(button("Clear Cache").on_press(Message::CacheCleared))
					.push(
						container(text(format!(
							"Size: {:.2} GBs",
							self.cache_size as f32 / 1_000_000_000.
						)))
						.padding(5),
					)
					.align_y(Alignment::Center),
			)),
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
