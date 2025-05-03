use super::*;

use std::path::PathBuf;

use emcore::instance::write_instance_history;
use getset::Getters;
use iced::{
	Task,
	futures::future,
	widget::{column, container, mouse_area, right_center, stack, text},
};
use tokio::{
	fs,
	io::{self, AsyncReadExt},
};
use tracing::{error, warn};

use crate::gui::widget::{button, icon, scrollable, tooltip};

pub enum Action {
	Loaded,
	Loading,
	None,
	OpenInstance(PathBuf),
	Task(Task<Message>),
}

#[derive(Debug, Clone, Getters)]
pub struct InstanceHistory {
	hover: Option<usize>,
	#[getset(get = "pub")]
	inner: Vec<(PathBuf, Option<String>)>,
}

#[derive(Debug, Clone)]
pub enum Message {
	HoverState(Option<usize>),
	Loaded,
	Loading,
	NewInstance,
	OpenDirectory(PathBuf),
	OpenInstance(PathBuf),
	Refresh(Vec<(PathBuf, Option<String>)>),
	RemoveInstance(usize),
}

impl InstanceHistory {
	pub fn new() -> (Self, Task<Message>) {
		let task = Task::done(Message::Loading)
			.chain(Task::perform(
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
						let Ok(settings) = toml::from_str::<emcore::instance::Settings>(&buffer)
						else {
							break 'read_toml (path, None);
						};
						break 'read_toml (path, settings.name);
					}))
					.await
				},
				Message::Refresh,
			))
			.chain(Task::done(Message::Loaded));

		(
			Self {
				hover: None,
				inner: Vec::new(),
			},
			task,
		)
	}

	pub fn update(&mut self, message: Message) -> Action {
		match message {
			Message::HoverState(hover) => self.hover = hover,
			Message::Loaded => return Action::Loaded,
			Message::Loading => return Action::Loading,
			Message::NewInstance => {
				return Action::Task(
					Task::done(Message::Loading).chain(
						Task::future(
							rfd::AsyncFileDialog::new()
								.set_title("Select game executable")
								.set_file_name("Exanima.exe")
								.add_filter("Exanima.exe", &["exe"])
								.pick_file(),
						)
						.and_then(|handle| {
							if let Some(path) = handle.path().parent() {
								Task::done(Message::OpenInstance(path.into()))
							} else {
								error!("failed to get parent directory of game executable");
								Task::none()
							}
						})
						.chain(Task::done(Message::Loaded)),
					),
				);
			}
			Message::OpenDirectory(path) => {
				let _ = open::that(path).map_err(|e| error!("{}", e));
			}
			Message::OpenInstance(path) => return Action::OpenInstance(path),
			Message::Refresh(history) => {
				self.inner = history;
			}
			Message::RemoveInstance(index) => {
				self.inner.remove(index);
				let history: Vec<_> = self
					.inner
					.clone()
					.into_iter()
					.map(|(path, _)| path)
					.collect();
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								write_instance_history(&history)
									.await
									.map_err(|e| error!("{}", e))
							})
							.discard(),
						)
						.chain(Task::done(Message::Loaded)),
				);
			}
		}

		Action::None
	}

	pub fn view(&self) -> Element<Message> {
		let content = container(
			column![
				button("New Instance")
					.on_press(Message::NewInstance)
					.width(Fill)
			]
			.extend(
				self.inner
					.iter()
					.enumerate()
					.map(|(i, (path, maybe_name))| {
						let instance_btn = tooltip(
							button(
								text(if let Some(name) = maybe_name {
									name.clone()
								} else {
									path.display().to_string()
								})
								.wrapping(text::Wrapping::None),
							)
							.on_press(Message::OpenInstance(path.clone()))
							.width(Fill),
							text(path.display().to_string()),
							tooltip::Position::Top,
						);

						let history_view: Element<_> = if let Some(hover) = self.hover
							&& hover == i
						{
							let open_btn = tooltip(
								button(icon::folder_open().size(16).center())
									.width(26)
									.height(26)
									.on_press(Message::OpenDirectory(path.clone()))
									.style(|theme, status| {
										let primary = button::primary(theme, status);
										match status {
											button::Status::Active => button::Style {
												background: Some(theme.palette().background.into()),
												..primary
											},
											_ => primary,
										}
									}),
								text("Open directory in file manager"),
								tooltip::Position::Top,
							);

							let remove_btn = tooltip(
								button(icon::trash().size(16).center())
									.width(26)
									.height(26)
									.on_press(Message::RemoveInstance(i))
									.style(|theme, status| {
										let danger = button::danger(theme, status);
										match status {
											button::Status::Active => button::Style {
												background: Some(theme.palette().background.into()),
												..danger
											},
											_ => danger,
										}
									}),
								text("Remove from history"),
								tooltip::Position::Top,
							);

							let controls = row![open_btn, remove_btn].spacing(4);

							stack![instance_btn, right_center(controls).padding([0, 4])].into()
						} else {
							instance_btn.into()
						};

						mouse_area(history_view)
							.on_enter(Message::HoverState(Some(i)))
							.on_exit(Message::HoverState(None))
							.into()
					}),
			)
			.spacing(1),
		)
		.width(400);

		container(
			scrollable(content)
				.direction(scrollable::Direction::Vertical(scrollable::Scrollbar::new())),
		)
		.center(Fill)
		.into()
	}

	pub fn title(&self) -> String {
		"Instance History".to_string()
	}
}
