//! iced

mod buffer;
mod log;
mod tab;
mod widget;

use std::{env, mem, path::PathBuf};

use emcore::{Error, Result, TomlError};
use getset::{Getters, WithSetters};
use iced::widget::pane_grid;
use iced::{
	Element, Event, Padding, Subscription, Task, Theme,
	advanced::widget as iced_widget,
	event, keyboard,
	widget::{container, pane_grid::Pane, responsive},
	window,
};
use iced_drop::zones_on_point;
use serde::{Deserialize, Serialize};
use tokio::{fs, io};
use tracing::{debug, error, info, instrument};

use buffer::{
	Buffer,
	instance::{self, Instance},
	instance_history::{self, InstanceHistory},
	logs::Logs,
	settings::{self, Settings},
};
use tab::TabManager;
use widget::toast::{self, Toast};

macro_rules! workspace_root {
	($path:expr) => {
		concat!(env!("WORKSPACE_ROOT"), $path)
	};
}

pub(crate) static ICON: &[u8] = include_bytes!(workspace_root!("/assets/images/corro.ico"));

// #[derive(Debug, Hash, PartialEq, Eq)]
// pub enum Icon {
// 	Play,
// 	X,
// }

// impl Icon {
// 	pub fn handle(&self) -> svg::Handle {
// 		macro_rules! handle {
// 			($path:expr) => {{
// 				static ICON: OnceLock<svg::Handle> = OnceLock::new();
// 				ICON.get_or_init(|| {
// 					svg::Handle::from_memory(include_bytes!(workspace_root!($path)))
// 				})
// 				.clone()
// 			}};
// 		}

// 		match self {
// 			Icon::Play => handle!("/assets/images/play.svg"),
// 			Icon::X => handle!("/assets/images/x.svg"),
// 		}
// 	}
// }

// impl From<Icon> for svg::Handle {
// 	fn from(value: Icon) -> Self {
// 		value.handle()
// 	}
// }

#[instrument(level = "trace")]
pub fn default_theme() -> Theme {
	Theme::Ferra
}

#[instrument(level = "trace")]
pub fn theme_from(val: &str) -> Option<Theme> {
	match val {
		"Light" => Some(Theme::Light),
		"Dark" => Some(Theme::Dark),
		"Dracula" => Some(Theme::Dracula),
		"Nord" => Some(Theme::Nord),
		"Solarized Light" => Some(Theme::SolarizedLight),
		"Solarized Dark" => Some(Theme::SolarizedDark),
		"Gruvbox Light" => Some(Theme::GruvboxLight),
		"Gruvbox Dark" => Some(Theme::GruvboxDark),
		"Catppuccin Latte" => Some(Theme::CatppuccinLatte),
		"Catppuccin Frappé" => Some(Theme::CatppuccinFrappe),
		"Catppuccin Macchiato" => Some(Theme::CatppuccinMacchiato),
		"Catppuccin Mocha" => Some(Theme::CatppuccinMocha),
		"Tokyo Night" => Some(Theme::TokyoNight),
		"Tokyo Night Storm" => Some(Theme::TokyoNightStorm),
		"Tokyo Night Light" => Some(Theme::TokyoNightLight),
		"Kanagawa Wave" => Some(Theme::KanagawaWave),
		"Kanagawa Dragon" => Some(Theme::KanagawaDragon),
		"Kanagawa Lotus" => Some(Theme::KanagawaLotus),
		"Moonfly" => Some(Theme::Moonfly),
		"Nightfly" => Some(Theme::Nightfly),
		"Oxocarbon" => Some(Theme::Oxocarbon),
		"Ferra" => Some(Theme::Ferra),
		_ => None,
	}
}

#[derive(Debug, Clone, Deserialize, Serialize, WithSetters)]
pub struct Config {
	#[getset(set_with = "pub")]
	theme: String,
}

impl Config {
	/// Attempt to deserialize and return GUI settings from the toml file stored
	/// in `emcore::DATA_DIR`.
	#[instrument(level = "trace")]
	async fn read_config() -> Result<Self> {
		let data_dir = emcore::data_dir()
			.ok_or(io::Error::new(
				io::ErrorKind::NotFound,
				"path does not exist",
			))
			.map_err(Error::msg("failed to get app data directory"))?;
		let config_path = data_dir.join(App::TOML);

		let buffer = fs::read_to_string(config_path)
			.await
			.map_err(Error::msg("failed to read into buffer for gui config"))?;
		info!("gui config file read into buffer");
		let gui_config = toml::from_str(&buffer)
			.map_err(TomlError::from)
			.map_err(Error::msg("failed to deserialize gui config from buffer"))?;
		info!("gui config deserialized from buffer");

		Ok(gui_config)
	}

	/// Returns a result from attempting to serialize the given config to
	/// `App::TOML` and then mutating `Root::config`.
	///
	/// # Errors
	///
	/// `App::config` will not be mutated if an error occurs.
	///
	/// Errors may be returned according to:
	///
	/// - `emcore::data_dir`
	/// - `toml::to_string`
	/// - `tokio::fs::write`
	#[instrument(level = "trace")]
	pub async fn write_config(self) -> Result<Self> {
		let toml_path = emcore::data_dir()
			.map(|p| p.join(App::TOML))
			.ok_or(io::Error::new(
				io::ErrorKind::NotFound,
				"path does not exist",
			))
			.map_err(Error::msg("failed to find app data directory"))?;
		let buffer = toml::to_string(&self)
			.map_err(TomlError::from)
			.map_err(Error::msg("failed to serialize gui config into buffer"))?;
		info!("gui config serialized to buffer");
		fs::write(toml_path, buffer)
			.await
			.map_err(Error::msg("failed to write gui config buffer into file"))?;
		info!("finished writing gui config to file");

		Ok(self)
	}
}

impl Default for Config {
	#[instrument(level = "trace")]
	fn default() -> Self {
		Theme::default();
		Self {
			theme: default_theme().to_string(),
		}
	}
}

#[derive(Debug, Clone, Getters, WithSetters)]
pub struct Root {
	#[getset(get = "pub", set_with = "pub")]
	config: Config,
	loading: bool,
	pub logs: Vec<log::Event>,
	#[getset(get = "pub")]
	theme: Theme,
}

impl Root {
	#[instrument(level = "trace")]
	fn new() -> (Self, Task<Message>) {
		let config_exists = emcore::data_dir().map(|p| p.join(App::TOML).is_file());
		let task = if let Some(is_file) = config_exists
			&& is_file
		{
			Task::future(Config::read_config())
				.map(|result| result.map_err(|e| error!("{}", e)))
				.and_then(|config| Task::done(Message::RefreshConfig(config)))
				.chain(Task::done(Message::Loaded))
		} else {
			Task::done(Message::Loaded)
		};
		(Self::default(), task)
	}
}

impl Default for Root {
	#[instrument(level = "trace")]
	fn default() -> Self {
		Self {
			config: Config::default(),
			loading: true,
			logs: Vec::new(),
			theme: default_theme(),
		}
	}
}

#[derive(Debug, Clone)]
pub enum Hotkey {
	ClosedTab,
	NewTab,
	RefreshTab,
}

#[derive(Debug)]
pub struct App {
	focus: Pane,
	tab_managers: pane_grid::State<TabManager>,
	toasts: Vec<Toast>,
	root: Root,
}

#[derive(Debug, Clone)]
enum Message {
	ClosedToast(usize),
	Loaded,
	Log(log::Event),
	HotkeyPressed(Hotkey),
	RefreshConfig(Config),
	ReplaceBuffer(Pane, iced_widget::Id, Buffer),
	Tab(tab::Message),
}

impl App {
	/// The name of the file responsible for storing user settings for the gui
	/// specifically. This is a child of `emcore::DATA_DIR`.
	pub const TOML: &'static str = "gui.toml";

	#[instrument(level = "trace")]
	pub(super) fn run() -> iced::Result {
		let image = image::load_from_memory(ICON).unwrap();
		let icon =
			window::icon::from_rgba(image.as_bytes().to_vec(), image.height(), image.width())
				.unwrap();

		iced::application(App::new, App::update, App::view)
			.font(include_bytes!(workspace_root!(
				"/assets/fonts/lucide/lucide.ttf"
			)))
			.subscription(App::subscription)
			.theme(App::theme)
			.title(App::title)
			.window(window::Settings {
				icon: Some(icon),
				..Default::default()
			})
			.run()
	}

	#[instrument(level = "trace")]
	fn new() -> (Self, Task<Message>) {
		let (root, root_task) = Root::new();
		let (tab_manager, buffer_task) = TabManager::new();
		let (tab_managers, focus) = pane_grid::State::new(tab_manager);
		(
			Self {
				focus,
				tab_managers,
				toasts: Vec::new(),
				root,
			},
			root_task.chain(
				buffer_task.map(move |message| Message::Tab(tab::Message::Buffer(focus, message))),
			),
		)
	}

	#[instrument(level = "trace")]
	fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::ClosedToast(index) => {
				if index < self.toasts.len() {
					self.toasts.remove(index);
				}
			}
			Message::Loaded => self.root.loading = false,
			Message::Log(event) => {
				self.root.logs.push(event.clone());
				if let Some((log, level)) = event.0.first()
					&& let Some(level) = level
					&& *level == tracing::Level::ERROR
				{
					self.toasts.push(Toast {
						title: "Error".to_string(),
						body: log.clone(),
						status: toast::Status::Danger,
						task_handle: None,
					});
				}
			}
			Message::HotkeyPressed(hotkey) => match hotkey {
				Hotkey::ClosedTab => {
					if let Some(tab_manager) = self.tab_managers.get(self.focus)
						&& !tab_manager.tabs.is_empty()
						&& let Some(tab_focus) = &tab_manager.focus
					{
						return Task::done(Message::Tab(tab::Message::ClosedTab(
							self.focus,
							tab_focus.clone(),
						)));
					}
				}
				Hotkey::NewTab => return Task::done(Message::Tab(tab::Message::NewTab)),
				Hotkey::RefreshTab => return Task::done(Message::Tab(tab::Message::RefreshTab)),
			},
			Message::RefreshConfig(config) => {
				self.root.config = config;
				if let Some(theme) = theme_from(&self.root.config.theme) {
					self.root.theme = theme;
				}
			}
			Message::ReplaceBuffer(pane, widget_id, buffer) => {
				if let Some(tab_manager) = self.tab_managers.get_mut(pane)
					&& let Some(tab) = tab_manager
						.tabs
						.iter_mut()
						.find(|tab| tab.widget_id == widget_id)
				{
					let path = if let Buffer::Instance(instance) = &buffer {
						instance.inner().path().clone()
					} else {
						PathBuf::new()
					};

					let _ = mem::replace(&mut tab.buffer, buffer);
					tab.loading = false;

					// FIX: condition doesn't work properly
					return if let Buffer::Instance(instance) = &tab.buffer
						&& !path.display().to_string().is_empty()
						&& *instance.inner().path() != path
					{
						let lock_path = instance
							.inner()
							.path()
							.join(emcore::Instance::DATA_DIR)
							.join(emcore::Instance::LOCK);
						Task::future(async move {
							if lock_path.is_file() {
								let _ = fs::remove_file(lock_path)
									.await
									.map(|_| info!("instance's lock file removed"))
									.map_err(Error::msg("failed to remove instance's lock file"))
									.map_err(|e| error!("{}", e));
							}
						})
						.discard()
					} else {
						Task::none()
					};
				}
			}
			Message::Tab(message) => match message {
				tab::Message::Buffer(pane, message) => {
					if let Some(tab_manager) = self.tab_managers.get_mut(pane)
						&& let Some(tab_focus) = &tab_manager.focus
						&& let Some(tab) = tab_manager
							.tabs
							.iter_mut()
							.find(|tab| tab.widget_id == *tab_focus)
					{
						let action = tab.buffer.update(message);
						match action {
							buffer::Action::Instance(action) => match action {
								instance::Action::Loaded => tab.loading = false,
								instance::Action::Loading => tab.loading = true,
								instance::Action::None => (),
								instance::Action::Task(task) => {
									return task.map(move |message| {
										Message::Tab(tab::Message::Buffer(
											pane,
											buffer::Message::Instance(message),
										))
									});
								}
							},
							buffer::Action::InstanceHistory(action) => match action {
								instance_history::Action::Loaded => tab.loading = false,
								instance_history::Action::Loading => tab.loading = true,
								instance_history::Action::None => (),
								instance_history::Action::OpenInstance(path) => {
									let widget_id = tab.widget_id.clone();
									return Task::perform(
										async move { Instance::with_path(&path).await },
										move |result| match result {
											Ok(instance) => (
												Message::ReplaceBuffer(
													pane,
													widget_id,
													instance.into(),
												),
												Task::none(),
											),
											Err(e) => {
												error!("{}", e);
												let (instance_history, task) =
													InstanceHistory::new();
												(
													Message::ReplaceBuffer(
														pane,
														widget_id,
														instance_history.into(),
													),
													task,
												)
											}
										},
									)
									.then(move |(message, task)| {
										Task::done(message).chain(task.map(move |message| {
											Message::Tab(tab::Message::Buffer(
												pane,
												buffer::Message::InstanceHistory(message),
											))
										}))
									});
								}
								instance_history::Action::Task(task) => {
									return task.map(move |message| {
										Message::Tab(tab::Message::Buffer(
											pane,
											buffer::Message::InstanceHistory(message),
										))
									});
								}
							},
							buffer::Action::Loaded => tab.loading = false,
							buffer::Action::Loading => tab.loading = true,
							buffer::Action::NewInstance => {
								let widget_id = tab.widget_id.clone();
								return Task::perform(
									async { Instance::new().await },
									move |result| match result {
										Some(new_instance) => (
											Message::ReplaceBuffer(
												pane,
												widget_id,
												new_instance.into(),
											),
											Task::none(),
										),
										None => {
											error!("failed to find last used instance in history");
											let (instance_history, task) = InstanceHistory::new();
											(
												Message::ReplaceBuffer(
													pane,
													widget_id,
													instance_history.into(),
												),
												task,
											)
										}
									},
								)
								.then(move |(message, task)| {
									Task::done(message).chain(task.map(move |message| {
										Message::Tab(tab::Message::Buffer(
											pane,
											buffer::Message::InstanceHistory(message),
										))
									}))
								});
							}
							buffer::Action::NewInstanceHistory => {
								let (instance_history, task) = InstanceHistory::new();
								return Task::done(Message::ReplaceBuffer(
									pane,
									tab.widget_id.clone(),
									instance_history.into(),
								))
								.chain(task.map(move |message| {
									Message::Tab(tab::Message::Buffer(
										pane,
										buffer::Message::InstanceHistory(message),
									))
								}));
							}
							buffer::Action::NewLogs => {
								return Task::done(Message::ReplaceBuffer(
									pane,
									tab.widget_id.clone(),
									Logs.into(),
								));
							}
							buffer::Action::NewSettings => {
								return Task::done(Message::ReplaceBuffer(
									pane,
									tab.widget_id.clone(),
									Settings.into(),
								));
							}
							buffer::Action::None => (),
							buffer::Action::OpenInstance(path) => {
								if let Buffer::Instance(instance) = &tab.buffer
									&& *instance.inner().path() == path
								{
									return Task::done(Message::Tab(tab::Message::RefreshTab));
								}

								let widget_id = tab.widget_id.clone();
								return Task::perform(
									async move { Instance::with_path(&path).await },
									move |result| match result {
										Ok(instance) => (
											Message::ReplaceBuffer(
												pane,
												widget_id,
												instance.into(),
											),
											Task::none(),
										),
										Err(e) => {
											error!("{}", e);
											let (instance_history, task) = InstanceHistory::new();
											(
												Message::ReplaceBuffer(
													pane,
													widget_id,
													instance_history.into(),
												),
												task,
											)
										}
									},
								)
								.then(move |(message, task)| {
									Task::done(message).chain(task.map(move |message| {
										Message::Tab(tab::Message::Buffer(
											pane,
											buffer::Message::InstanceHistory(message),
										))
									}))
								});
							}
							buffer::Action::Settings(action) => match action {
								settings::Action::None => {}
								settings::Action::ThemeSelected(theme) => {
									let config =
										self.root.config.clone().with_theme(theme.to_string());
									return Task::future(config.write_config())
										.map(|result| result.map_err(|e| error!("{}", e)))
										.and_then(|config| {
											Task::done(Message::RefreshConfig(config))
										})
										.chain(Task::done(Message::Loaded));
								}
							},
							buffer::Action::Task(task) => {
								return task.map(move |message| {
									Message::Tab(tab::Message::Buffer(pane, message))
								});
							}
						}
					} else {
						debug!("failed to find currently focused tab, doing nothing")
					}
				}
				tab::Message::ClosedPane(pane) => {
					if self.tab_managers.close(pane).is_some() {
						info!("closed pane of tabs");
					};
				}
				tab::Message::ClosedTab(pane, tab_id) => {
					let pane_count = self.tab_managers.len();
					if let Some(tab_manager) = self.tab_managers.get_mut(pane)
						&& let Some(index) =
							tab_manager.tabs.iter().enumerate().find_map(|(i, tab)| {
								if tab.widget_id == tab_id {
									Some(i)
								} else {
									None
								}
							}) {
						let tab = tab_manager.tabs.remove(index);

						let buffer_task = if let Buffer::Instance(instance) = tab.buffer {
							let lock_path = instance
								.inner()
								.path()
								.join(emcore::Instance::DATA_DIR)
								.join(emcore::Instance::LOCK);
							Task::future(async move {
								if lock_path.is_file() {
									let _ = fs::remove_file(lock_path)
										.await
										.map(|_| info!("instance's lock file removed"))
										.map_err(Error::msg(
											"failed to remove instance's lock file",
										))
										.map_err(|e| error!("{}", e));
								}
							})
							.discard()
						} else {
							Task::none()
						};

						if pane_count > 1 && tab_manager.tabs.is_empty() {
							return Task::batch([
								Task::done(Message::Tab(tab::Message::ClosedPane(pane))),
								buffer_task,
							]);
						} else if tab_manager.tabs.is_empty() {
							return Task::batch([
								Task::done(Message::Tab(tab::Message::NewTab)),
								buffer_task,
							]);
						}

						let offset = if !tab_manager.tabs.is_empty()
							&& index > 0 && index == tab_manager.tabs.len()
						{
							index - 1
						} else {
							index
						};
						tab_manager.focus = tab_manager
							.tabs
							.get(offset)
							.map(|tab| tab.widget_id.clone());

						return buffer_task;
					} else {
						debug!(
							"failed to find currently focused pane to close tab in, doing nothing"
						)
					}
				}
				tab::Message::ClickedPane(pane) => self.focus = pane,
				tab::Message::ClickedTab(pane, tab_id) => {
					if let Some(tab_manager) = self.tab_managers.get_mut(pane) {
						tab_manager.focus = Some(tab_id);
					} else {
						debug!(
							"failed to find currently focused pane to focus on selected tab, doing nothing"
						)
					}
				}
				tab::Message::DockTab(pane) => {}
				tab::Message::DraggedPane(event) => match event {
					pane_grid::DragEvent::Picked { .. } => (),
					pane_grid::DragEvent::Dropped { pane, target } => {
						self.tab_managers.drop(pane, target)
					}
					pane_grid::DragEvent::Canceled { .. } => (),
				},
				tab::Message::DraggedTab(point, _bounds) => {
					if let Some(tab_manager) = self.tab_managers.get(self.focus)
						&& let Some(tab_focus) = &tab_manager.focus
						&& let Some(tab) = tab_manager
							.tabs
							.iter()
							.find(|tab| tab.widget_id == *tab_focus)
					{
						let options = self
							.tab_managers
							.panes
							.iter()
							.flat_map(|(_, tab_manager)| {
								tab_manager.tabs.iter().filter_map(|option| {
									if tab.widget_id != option.widget_id {
										Some(option.widget_id.clone())
									} else {
										None
									}
								})
							})
							.collect();

						return zones_on_point(
							|zones| Message::Tab(tab::Message::OverTab(zones)),
							point,
							Some(options),
							None,
						);
					}
				}
				tab::Message::DraggedTabCanceled => {
					self.tab_managers
						.panes
						.iter_mut()
						.for_each(|(_, tab_manager)| {
							tab_manager.over = None;
						})
				}
				tab::Message::DroppedTab(point, _bounds) => {
					if let Some(tab_manager) = self.tab_managers.get(self.focus)
						&& let Some(tab_focus) = &tab_manager.focus
						&& let Some(tab) = tab_manager
							.tabs
							.iter()
							.find(|tab| tab.widget_id == *tab_focus)
					{
						let options = self
							.tab_managers
							.panes
							.iter()
							.flat_map(|(_, tab_manager)| {
								tab_manager.tabs.iter().filter_map(|option| {
									if tab.widget_id != option.widget_id {
										Some(option.widget_id.clone())
									} else {
										None
									}
								})
							})
							.collect();
						return zones_on_point(
							|zones| Message::Tab(tab::Message::ReorderTabs(zones)),
							point,
							Some(options),
							None,
						);
					}
				}
				tab::Message::EnteredTabRegion(pane, widget_id) => {
					if let Some(tab_manager) = self.tab_managers.get_mut(pane) {
						tab_manager.hover = Some(widget_id)
					}
				}
				tab::Message::ExitedTabRegion(pane, widget_id) => {
					if let Some(tab_manager) = self.tab_managers.get_mut(pane)
						&& let Some(hovered_widget_id) = &tab_manager.hover
						&& widget_id == *hovered_widget_id
					{
						tab_manager.hover = None;
					}
				}
				tab::Message::NewPane => {
					let (tab_manager, buffer_task) = TabManager::new();
					if let Some((new_pane, _)) =
						self.tab_managers
							.split(pane_grid::Axis::Vertical, self.focus, tab_manager)
					{
						info!("new pane created");
						return buffer_task.map(move |message| {
							Message::Tab(tab::Message::Buffer(new_pane, message))
						});
					};
				}
				tab::Message::NewTab => {
					let pane = self.focus;
					if let Some(tab_manager) = self.tab_managers.get_mut(pane) {
						let (instance_history, task) = InstanceHistory::new();
						let tab = tab::Tab::new(instance_history.into());
						tab_manager.focus = Some(tab.widget_id.clone());
						tab_manager.tabs.push(tab);
						info!("new tab created");
						return task.map(move |message| {
							Message::Tab(tab::Message::Buffer(
								pane,
								buffer::Message::InstanceHistory(message),
							))
						});
					} else {
						debug!(
							"failed to find currently focused pane to create new tab in, doing nothing"
						)
					}
				}
				tab::Message::OverTab(zones) => {
					let Some((widget_id, _)) = zones.into_iter().next() else {
						return Task::none();
					};
					self.tab_managers
						.panes
						.iter_mut()
						.for_each(|(_, tab_manager)| {
							let maybe_tab = tab_manager
								.tabs
								.iter()
								.find(|tab| tab.widget_id == widget_id);
							tab_manager.over = maybe_tab.map(|tab| tab.widget_id.clone());
						});
				}
				tab::Message::RefreshTab => {
					if let Some(tab_manager) = self.tab_managers.get_mut(self.focus)
						&& let Some(tab_focus) = &tab_manager.focus
						&& let Some(tab) = tab_manager
							.tabs
							.iter()
							.find(|tab| tab.widget_id == *tab_focus)
					{
						match &tab.buffer {
							Buffer::Instance(instance) => {
								let pane = self.focus;
								let path = instance.inner().path().clone();
								let widget_id = tab.widget_id.clone();
								let buffer = tab.buffer.clone();
								let lock_path = instance
									.inner()
									.path()
									.join(emcore::Instance::DATA_DIR)
									.join(emcore::Instance::LOCK);
								return Task::future(async move {
									if lock_path.is_file() {
										let _ = fs::remove_file(lock_path)
											.await
											.map(|_| info!("instance's lock file removed"))
											.map_err(Error::msg(
												"failed to remove instance's lock file",
											))
											.map_err(|e| error!("{}", e));
									}
								})
								.discard()
								.chain(Task::perform(
									async move {
										let Ok(new_instance) = Instance::with_path(&path)
											.await
											.map_err(|e| error!("{}", e))
										else {
											return None;
										};
										Some(new_instance)
									},
									move |result| {
										if let Some(instance) = result {
											Message::ReplaceBuffer(pane, widget_id, instance.into())
										} else {
											Message::ReplaceBuffer(pane, widget_id, buffer)
										}
									},
								));
							}
							Buffer::InstanceHistory(_instance_history) => {
								let pane = self.focus;
								let (instance_history, task) = InstanceHistory::new();
								return Task::done(Message::ReplaceBuffer(
									self.focus,
									tab.widget_id.clone(),
									instance_history.into(),
								))
								.chain(task.map(move |message| {
									Message::Tab(tab::Message::Buffer(
										pane,
										buffer::Message::InstanceHistory(message),
									))
								}));
							}
							Buffer::Logs(_logs) => {
								return Task::done(Message::ReplaceBuffer(
									self.focus,
									tab.widget_id.clone(),
									Logs.into(),
								));
							}
							Buffer::Settings(_settings) => {
								return Task::done(Message::ReplaceBuffer(
									self.focus,
									tab.widget_id.clone(),
									Settings.into(),
								));
							}
						}
					} else {
						debug!("failed to find currently focused tab to refresh, doing nothing")
					}
				}
				tab::Message::ReorderTabs(zones) => {
					let Some((widget_id, _)) = zones.first() else {
						return Task::none();
					};

					let Some((to_pane, to_index)) =
						self.tab_managers
							.panes
							.iter()
							.find_map(|(pane, tab_manager)| {
								tab_manager
									.tabs
									.iter()
									.enumerate()
									.find_map(|(i, tab)| {
										if tab.widget_id == *widget_id {
											Some(i)
										} else {
											None
										}
									})
									.map(|index| (*pane, index))
							})
					else {
						return Task::none();
					};

					if self.focus == to_pane {
						let Some(tab_manager) = self.tab_managers.get_mut(self.focus) else {
							return Task::none();
						};

						let Some(tab_focus) = &tab_manager.focus else {
							return Task::none();
						};

						let Some(index) =
							tab_manager.tabs.iter().enumerate().find_map(|(i, tab)| {
								if tab.widget_id == *tab_focus {
									Some(i)
								} else {
									None
								}
							})
						else {
							return Task::none();
						};

						let tab = tab_manager.tabs.remove(index);
						tab_manager.hover = Some(tab.widget_id.clone());
						tab_manager.tabs.insert(to_index, tab);
						// tab_manager.focus = Some(to_index);
					} else {
						let Some(tab_manager) = self.tab_managers.get_mut(self.focus) else {
							return Task::none();
						};

						let Some(tab_focus) = &tab_manager.focus else {
							return Task::none();
						};

						let Some(index) =
							tab_manager.tabs.iter().enumerate().find_map(|(i, tab)| {
								if tab.widget_id == *tab_focus {
									Some(i)
								} else {
									None
								}
							})
						else {
							return Task::none();
						};

						// TODO: refactor to not lose data when returned early
						// FIX: removing last tab in states sets tab.focus out of bounds
						let tab = tab_manager.tabs.remove(index);

						let offset = if !tab_manager.tabs.is_empty()
							&& index > 0 && index == tab_manager.tabs.len()
						{
							index - 1
						} else {
							index
						};
						tab_manager.focus = tab_manager
							.tabs
							.get(offset)
							.map(|tab| tab.widget_id.clone());

						// // BUG: overflow
						// if from_tab_focus == from_tab_manager.tabs.len() - 1 {
						// 	from_tab_manager.focus = Some(from_tab_focus - 1);
						// }

						let task =
							if tab_manager.tabs.is_empty() && self.tab_managers.panes.len() > 1 {
								Task::done(Message::Tab(tab::Message::ClosedPane(self.focus)))
							} else {
								Task::none()
							};

						let Some(to_tab_manager) = self.tab_managers.get_mut(to_pane) else {
							return Task::none();
						};

						// let Some(to_tab_focus) = &to_tab_manager.focus else {
						// 	return Task::none();
						// };

						// if to_tab_manager.tabs.get(to_tab_focus).is_none() {
						// 	return Task::none();
						// };

						to_tab_manager.hover = Some(tab.widget_id.clone());
						to_tab_manager.tabs.insert(to_index, tab);
						// to_tab_manager.focus = Some(to_index);
						return task;
					}
				}
				tab::Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
					self.tab_managers.resize(split, ratio)
				}
			},
		}

		Task::none()
	}

	#[instrument(level = "trace")]
	fn view(&self) -> Element<Message> {
		let pane_grid: Element<_> = pane_grid(&self.tab_managers, |pane, tab, _is_maximized| {
			let title_bar = pane_grid::TitleBar::new({
				container(responsive(move |size| {
					tab.view_header(&self.tab_managers, pane, size)
				}))
				.height(38)
			})
			.padding(Padding::default().left(32));

			pane_grid::Content::new(tab.view(&self.tab_managers, pane, &self.root))
				.title_bar(title_bar)
		})
		.spacing(1)
		.on_click(tab::Message::ClickedPane)
		.on_drag(tab::Message::DraggedPane)
		.on_resize(10, tab::Message::Resized)
		.into();

		let content = container(pane_grid.map(Message::Tab)).style(|theme: &Theme| {
			let ext_palette = theme.extended_palette();
			container::Style {
				background: Some(ext_palette.background.weak.color.into()),
				text_color: Some(ext_palette.background.weak.text),
				..Default::default()
			}
		});

		toast::Manager::new(content, &self.toasts, Message::ClosedToast)
			.timeout(10)
			.into()
	}

	#[instrument(level = "trace")]
	fn subscription(&self) -> Subscription<Message> {
		let logs = Subscription::run(log::stream).map(Message::Log);

		let events = event::listen_with(|event, _status, _id| {
			if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
				if modifiers == keyboard::Modifiers::CTRL {
					if key == keyboard::Key::Character("w".into()) {
						return Some(Message::HotkeyPressed(Hotkey::ClosedTab));
					} else if key == keyboard::Key::Character("t".into()) {
						return Some(Message::HotkeyPressed(Hotkey::NewTab));
					} else if key == keyboard::Key::Character("r".into()) {
						return Some(Message::HotkeyPressed(Hotkey::RefreshTab));
					}
				}
			}

			None
		});

		Subscription::batch([logs, events])
	}

	#[instrument(level = "trace")]
	fn title(&self) -> String {
		let mut title = format!("Exanima Modding Toolkit v{}", env!("CARGO_PKG_VERSION"));

		if let Some(tab_manager) = self.tab_managers.get(self.focus)
			&& let Some(tab_focus) = &tab_manager.focus
			&& let Some(tab) = tab_manager
				.tabs
				.iter()
				.find(|tab| tab.widget_id == *tab_focus)
		{
			title.push_str(" - ");
			title.push_str(&tab.buffer.title());
		}

		title
	}

	#[instrument(level = "trace")]
	fn theme(&self) -> Theme {
		self.root.theme.clone()
	}
}
