use super::*;

use std::path::PathBuf;

use emtk_core::instance::write_instance_history;
use getset::Getters;
use iced::{
	Task, Theme,
	futures::future,
	widget::{
		column, container, horizontal_rule, mouse_area, responsive, right_center, scrollable,
		stack, text,
	},
};
use tokio::{
	fs,
	io::{self, AsyncReadExt},
};
use tracing::{error, instrument, warn};

use crate::gui::widget::{button, icon, tooltip};

pub enum Action {
	Loaded,
	Loading,
	None,
	OpenInstance(PathBuf),
	Task(Task<Message>),
}

#[derive(Debug, Getters)]
pub struct InstanceHistory {
	hover: Option<usize>,
	#[getset(get = "pub")]
	inner: Vec<(PathBuf, Option<String>)>,
}

#[derive(Debug, Clone)]
pub enum Message {
	EnteredBtnRegion(Option<usize>),
	ExitedBtnRegion(Option<usize>),
	Loaded,
	Loading,
	NewInstance,
	OpenDirectory(PathBuf),
	OpenInstance(PathBuf),
	Refresh(Vec<(PathBuf, Option<String>)>),
	RemoveInstance(usize),
}

impl InstanceHistory {
	#[instrument(level = "trace")]
	pub fn new() -> (Self, Task<Message>) {
		let task = Task::done(Message::Loading)
			.chain(Task::perform(
				async {
					let history = match emtk_core::instance::history().await {
						Ok(instance_history) => instance_history,
						Err(e) => {
							warn!("{}", e);
							Vec::new()
						}
					};

					future::join_all(history.into_iter().map(async |path| 'read_toml: {
						let Ok(file) = fs::File::open(
							path.join(emtk_core::Instance::DATA_DIR)
								.join(emtk_core::Instance::TOML),
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
						let Ok(settings) = toml::from_str::<emtk_core::instance::Settings>(&buffer)
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

	#[instrument(level = "trace")]
	pub fn update(&mut self, message: Message) -> Action {
		match message {
			Message::EnteredBtnRegion(hover) => self.hover = hover,
			Message::ExitedBtnRegion(hover) => {
				if self.hover == hover {
					self.hover = None
				}
			}
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
				if path.is_dir() {
					let _ = open::that(path).map_err(|e| error!("{}", e));
				} else {
					error!("path to directory does not exist");
				}
			}
			Message::OpenInstance(path) => return Action::OpenInstance(path),
			Message::Refresh(history) => {
				self.inner = history;
			}
			Message::RemoveInstance(index) => {
				dbg!(&index);
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

	#[instrument(level = "trace")]
	pub fn view(&self) -> Element<'_, Message> {
		let history_len = self.inner.len() as u32;
		let history_btn_size = 54;
		let control_icon_size = 18;
		let control_btn_size = 36;
		responsive(move |size| {
			let content =
				container(
					column![tooltip(
						button(
							row![
								icon::square_arrow_out_up_right()
									.size(17)
									.center()
									.height(Fill),
								text("New Instance").size(20).center().height(Fill)
							]
							.spacing(5)
						)
						.on_press(Message::NewInstance)
						.width(Fill)
						.height(history_btn_size),
						text("Open file dialog"),
						tooltip::Position::FollowCursor
					)]
					.extend(self.inner.iter().enumerate().rev().map(
						|(i, (path, maybe_name))| {
							let instance_btn = mouse_area(tooltip(
								button(column![
									text(if let Some(name) = maybe_name {
										name.clone()
									} else {
										path.file_name().unwrap().display().to_string()
									})
									.size(20)
									.center()
									.wrapping(text::Wrapping::None),
									text(path.display().to_string()).size(14).center().style(
										move |theme: &Theme| {
											let ext_palette = theme.extended_palette();
											let color = Some(
												if self.hover == Some(i) {
													ext_palette.primary.base.text
												} else {
													ext_palette.background.base.text
												}
												.scale_alpha(0.5),
											);
											text::Style { color }
										}
									)
								])
								.on_press(Message::OpenInstance(path.clone()))
								.width(Fill)
								.height(history_btn_size),
								text("Open instance"),
								tooltip::Position::FollowCursor,
							))
							.on_enter(Message::EnteredBtnRegion(Some(i)))
							.on_exit(Message::ExitedBtnRegion(Some(i)));

							let history_view: Element<_> = if let Some(hover) = self.hover
								&& hover == i
							{
								let open_btn = tooltip(
									button(icon::folder_open().size(control_icon_size).center())
										.width(control_btn_size)
										.height(control_btn_size)
										.on_press(Message::OpenDirectory(path.clone()))
										.style(|theme, status| {
											let primary = button::primary(theme, status);
											match status {
												button::Status::Active => button::Style {
													background: Some(
														theme.palette().background.into(),
													),
													..primary
												},
												_ => primary,
											}
										}),
									text("Open directory in file manager"),
									tooltip::Position::Top,
								);

								let remove_btn = tooltip(
									button(icon::trash().size(control_icon_size).center())
										.width(control_btn_size)
										.height(control_btn_size)
										.on_press(Message::RemoveInstance(i))
										.style(|theme, status| {
											let danger = button::danger(theme, status);
											match status {
												button::Status::Active => button::Style {
													background: Some(
														theme.palette().background.into(),
													),
													..danger
												},
												_ => danger,
											}
										}),
									text("Remove from history list"),
									tooltip::Position::Top,
								);

								let controls = row![open_btn, remove_btn].spacing(8);

								// FIX: controls break hover state sync between mouse_area and button::Status::Hovered
								stack![instance_btn, right_center(controls).padding([0, 8])].into()
							} else {
								instance_btn.into()
							};

							column![horizontal_rule(5), history_view,].into()
						},
					)),
				);

			if (history_len * (history_btn_size + 5)) as f32 >= size.height {
				scrollable(content.padding(Padding::default().right(10)))
					.width(Fill)
					.height(Fill)
					.into()
			} else {
				content.into()
			}
		})
		.into()
	}

	#[instrument(level = "trace")]
	pub fn title(&self) -> String {
		"Instance History".to_string()
	}
}
