use super::*;

use std::path::PathBuf;

use getset::Getters;
use iced::{
	futures::future,
	widget::{column, container, text},
	Task,
};
use tokio::{
	fs,
	io::{self, AsyncReadExt},
};
use tracing::warn;

use crate::gui::widget::{button, tooltip};

pub enum Action {
	Loaded,
	Loading,
	OpenInstance(PathBuf),
	Task(Task<Message>),
}

#[derive(Debug, Clone, Getters)]
pub struct InstanceHistory {
	#[getset(get = "pub")]
	inner: Vec<(PathBuf, Option<String>)>,
}

#[derive(Debug, Clone)]
pub enum Message {
	Loaded,
	Loading,
	NewInstance,
	OpenInstance(PathBuf),
	Refresh(Vec<(PathBuf, Option<String>)>),
}

impl InstanceHistory {
	pub fn new() -> (Self, Task<Message>) {
		let task = Task::perform(
			async {
				let history = match emcore::instance::history().await {
					Ok(instance_history) => instance_history,
					Err(e) => {
						warn!("{}", e);
						Vec::new()
					}
				};

				future::join_all(history.into_iter().map(async |path| 'read_toml: {
					let Ok(file) = fs::File::open(
						path.join(emcore::Instance::DATA_DIR)
							.join(emcore::Instance::TOML),
					)
					.await
					else {
						break 'read_toml (path, None);
					};
					let mut reader = io::BufReader::new(file);
					let mut buffer = String::new();
					if reader.read_to_string(&mut buffer).await.is_err() {
						break 'read_toml (path, None);
					};
					let Ok(settings) = toml::from_str::<emcore::instance::Settings>(&buffer) else {
						break 'read_toml (path, None);
					};
					break 'read_toml (path, settings.name);
				}))
				.await
			},
			Message::Refresh,
		);

		(Self { inner: Vec::new() }, task)
	}

	pub fn update(&mut self, message: Message) -> Action {
		match message {
			Message::Loaded => Action::Loaded,
			Message::Loading => Action::Loading,
			Message::NewInstance => Action::Task(
				Task::done(Message::Loading).chain(
					Task::future(rfd::AsyncFileDialog::new().pick_folder())
						.and_then(|handle| Task::done(Message::OpenInstance(handle.path().into())))
						.chain(Task::done(Message::Loaded)),
				),
			),
			Message::OpenInstance(path) => Action::OpenInstance(path),
			Message::Refresh(history) => {
				self.inner = history;
				Action::Loaded
			}
		}
	}

	pub fn view(&self) -> Element<Message> {
		let content = column![button("New Instance").on_press(Message::NewInstance)].extend(
			self.inner.iter().map(|(path, maybe_name)| {
				if let Some(name) = maybe_name {
					tooltip(
						button(text(name)).on_press(Message::OpenInstance(path.clone())),
						text(path.display().to_string()),
						tooltip::Position::Top,
					)
					.into()
				} else {
					button(text(path.display().to_string()))
						.on_press(Message::OpenInstance(path.clone()))
						.into()
				}
			}),
		);

		container(content).center(Fill).into()
	}

	pub fn title(&self) -> String {
		"Instance History".to_string()
	}
}
