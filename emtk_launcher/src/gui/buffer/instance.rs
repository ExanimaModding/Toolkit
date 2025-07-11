use std::{
	env, fmt,
	path::{Path, PathBuf},
};

use anyhow::anyhow;
use emtk_core::{Error, TomlError, instance, plugin, profile};
use getset::Getters;
use iced::{
	Alignment, Border, Element, Fill, Font, Length, Point, Rectangle, Renderer, Shadow, Task,
	Theme,
	advanced::widget as iced_widget,
	widget::{
		Button, Space, Text, center_x, checkbox, column, horizontal_rule, horizontal_space,
		markdown, pick_list, responsive, row, rule, scrollable, text, text_editor, text_input,
		vertical_rule,
	},
};
use iced_drop::zones_on_point;
use iced_split::{Direction, Split};
use iced_table::table;
use tokio::{fs, io};
use tracing::{error, info, instrument};

use crate::gui::{
	Root,
	widget::{button, container, icon, tooltip},
};

/// Width of the markdown content.
const MD_WIDTH: f32 = 896.;

pub enum Action {
	InitFailed,
	Loaded,
	Loading,
	None,
	Task(Task<Message>),
}

/// The types of columns found in the load order table.
#[derive(Debug, Clone)]
pub enum ColumnKind {
	Name,
	Version,
	Priority,
}

/// The view state of each column in the load order table.
#[derive(Debug, Clone)]
pub struct Column {
	kind: ColumnKind,
	width: f32,
	resize_offset: Option<f32>,
}

impl Column {
	#[instrument(level = "trace")]
	pub fn new(kind: ColumnKind) -> Self {
		let width = match kind {
			ColumnKind::Name => 400.,
			ColumnKind::Version => 150.,
			ColumnKind::Priority => 60.,
		};

		Self {
			kind,
			width,
			resize_offset: None,
		}
	}
}

impl<'a> table::Column<'a, Message, Theme, Renderer> for Column {
	type Row = Row;

	#[instrument(level = "trace")]
	fn header(&'a self, _col_index: usize) -> Element<'a, Message> {
		let content = match self.kind {
			ColumnKind::Name => "Name",
			ColumnKind::Version => "Version",
			ColumnKind::Priority => "Priority",
		};

		text(content).into()
	}

	#[instrument(level = "trace")]
	fn cell(&'a self, _col_index: usize, row_index: usize, row: &'a Row) -> Element<'a, Message> {
		let plugin_valid = row.plugin.display_name.is_some() && row.plugin.version.is_some();

		let content: Element<_> = match self.kind {
			ColumnKind::Name => {
				let content = tooltip(
					row![
						checkbox(String::new(), row.plugin.enabled,)
							.on_toggle(move |enabled| Message::EntryToggled(row_index, enabled))
							.icon(checkbox::Icon {
								font: Font::with_name("lucide"),
								code_point: '\u{E805}',
								size: None,
								line_height: text::LineHeight::Relative(1.),
								shaping: text::Shaping::Basic,
							}),
						text(
							row.plugin
								.display_name
								.clone()
								.unwrap_or(row.plugin_id.to_string())
						)
					],
					text(row.plugin_id.to_string()),
					tooltip::Position::Top,
				);

				if plugin_valid {
					content.into()
				} else {
					row![
						content,
						tooltip(
							icon::info().size(16).center().style(text::danger),
							"Missing or invalid manifest",
							tooltip::Position::Top
						),
					]
					.spacing(8)
					.into()
				}
			}
			ColumnKind::Version => {
				text(row.plugin.version.to_owned().unwrap_or("? ? ?".to_string())).into()
			}
			ColumnKind::Priority => text(row.plugin.priority).into(),
		};

		let layout = container(content).style(move |theme: &Theme| {
			let default = container::Style {
				text_color: Some(theme.palette().text),
				..container::Style::default()
			};
			if plugin_valid {
				default
			} else {
				container::Style {
					text_color: Some(theme.palette().text.scale_alpha(0.5)),
					..default
				}
			}
		});

		layout.width(Fill).into()
	}

	#[instrument(level = "trace")]
	fn footer(&'a self, _col_index: usize, rows: &'a [Row]) -> Option<Element<'a, Message>> {
		match self.kind {
			ColumnKind::Name => Some(
				tooltip(
					button(icon::plus().size(12).center())
						.on_press(Message::NewPlugin)
						.width(28)
						.height(28),
					text("Create new plugin"),
					tooltip::Position::Top,
				)
				.into(),
			),
			ColumnKind::Version => None,
			ColumnKind::Priority => Some(
				container(text(rows.len()))
					.align_y(Alignment::Center)
					.into(),
			),
		}
	}

	#[instrument(level = "trace")]
	fn width(&self) -> f32 {
		self.width
	}

	#[instrument(level = "trace")]
	fn resize_offset(&self) -> Option<f32> {
		self.resize_offset
	}
}

/// The view state of a plugin in the load order table.
#[derive(Debug, Clone)]
pub struct Row {
	widget_id: iced_widget::Id,
	plugin_id: plugin::Id,
	plugin: profile::LoadOrderEntry,
}

impl iced_table::WithId for Row {
	#[instrument(level = "trace")]
	fn id(&self) -> iced::advanced::graphics::core::widget::Id {
		self.widget_id.clone()
	}
}

/// The view state of the load order.
#[derive(Debug, Clone)]
pub struct Table {
	body: scrollable::Id,
	columns: Vec<Column>,
	focus_row: Option<usize>,
	header: scrollable::Id,
	over: Option<iced_widget::Id>,
	rows: Vec<Row>,
}

impl Table {
	#[instrument(level = "trace")]
	pub fn new(instance: &emtk_core::Instance) -> Self {
		let mut table = Self::default();
		table.refresh(instance.profile().load_order().clone());
		table
	}

	/// Fills the table's rows with the current profile's load order. This can be
	/// used in combination with `Instance::refresh` to fully update the load order.
	#[instrument(level = "trace")]
	pub fn refresh(&mut self, load_order: profile::LoadOrder) -> &mut Self {
		let mut load_order: Vec<_> = load_order.into_iter().collect();
		load_order.sort_by(|(_, a), (_, b)| a.priority.cmp(&b.priority));
		self.rows = load_order
			.into_iter()
			.map(|(plugin_id, plugin)| Row {
				widget_id: iced_widget::Id::unique(),
				plugin_id,
				plugin,
			})
			.collect();
		self
	}
}

impl Default for Table {
	fn default() -> Self {
		Self {
			body: scrollable::Id::unique(),
			columns: vec![
				Column::new(ColumnKind::Name),
				Column::new(ColumnKind::Version),
				Column::new(ColumnKind::Priority),
			],
			focus_row: None,
			header: scrollable::Id::unique(),
			over: None,
			rows: Vec::new(),
		}
	}
}

/// The different states the instance view state can be in.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum State {
	#[default]
	Default,
	ProfileForm,
}

/// The view state of a plugin in the instance's load order.
#[derive(Debug, Default)]
pub enum Plugin {
	Editor {
		/// Text editor content
		content: text_editor::Content,
		/// Unsaved changes
		is_dirty: bool,
		/// Preview mode
		is_preview: bool,
		/// Preview content made from the text editor content
		preview: Vec<markdown::Item>,
	},
	Markdown(Vec<markdown::Item>),
	#[default]
	None,
	Settings(Option<plugin::Settings>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum MarkdownKind {
	Changelog,
	License,
	#[default]
	Readme,
}

impl fmt::Display for MarkdownKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			MarkdownKind::Changelog => f.write_str("changelog"),
			MarkdownKind::License => f.write_str("license"),
			MarkdownKind::Readme => f.write_str("readme"),
		}
	}
}

/// The view state of the instance.
#[derive(Debug, Getters)]
pub struct Instance {
	#[getset(get = "pub")]
	inner: emtk_core::Instance,
	is_plugin_maximized: bool,
	markdown_kind: Option<MarkdownKind>,
	plugin: Plugin,
	profiles: Vec<PathBuf>,
	profile_form_input: String,
	split: f32,
	state: State,
	table: Table,
}

#[derive(Debug, Clone)]
pub enum Message {
	ClickedRow(usize),
	DraggedRow(Point, Rectangle),
	DraggedRowCanceled,
	DroppedRow(Point, Rectangle),
	EntryToggled(usize, bool),
	Init(emtk_core::Instance),
	InitFailed,
	Launch,
	LinkClicked(markdown::Url),
	Loaded,
	Loading,
	NewPlugin,
	OverRow(Vec<(iced_widget::Id, Rectangle)>),
	PluginClosed,
	PluginDirectoryOpened(PathBuf),
	PluginEditMarkdown,
	PluginMaximizeToggled,
	PluginMarkdownChanged((MarkdownKind, String)),
	PluginReadMarkdown(MarkdownKind),
	PluginReadSettings,
	PluginSaveMarkdown,
	PluginSettingsChanged(toml::Value),
	PluginSettingsLoaded(Option<plugin::Settings>),
	ProfileChanged(emtk_core::Profile),
	ProfileDeleted,
	ProfileFormInputChanged(String),
	ProfileFormSubmitted,
	Profiles(Vec<PathBuf>),
	ProfileSelected(String),
	RefreshProfiles,
	ReorderRows(Vec<(iced_widget::Id, Rectangle)>),
	SettingsPressed,
	SplitDragged(f32),
	StateChanged(State),
	TableResized,
	TableResizing(usize, f32),
	TableSyncHeader(scrollable::AbsoluteOffset),
	TextEditorLoaded(String),
	TextEditorAction(text_editor::Action),
}

impl Instance {
	#[instrument(level = "trace")]
	pub fn new() -> (Self, Task<Message>) {
		(
			Self::default(),
			Task::done(Message::Loading)
				.chain(
					Task::future(async move {
						let instance_history = instance::history().await?;
						let Some(instance_path) = instance_history.last() else {
							return Err(Error::new(
								anyhow!("instance history is empty"),
								"failed to initialize instance",
							));
						};

						emtk_core::Instance::with_path(instance_path)?.build().await
					})
					.map(|result| {
						result
							.map(Message::Init)
							.map_err(|e| error!("{}", e))
							.unwrap_or(Message::InitFailed)
					}),
				)
				.chain(Task::done(Message::Loading)),
		)
	}

	#[instrument(level = "trace")]
	pub fn with_path(path: &Path) -> (Self, Task<Message>) {
		let path = path.to_owned();
		(
			Self::default(),
			Task::done(Message::Loading)
				.chain(
					Task::future(async move { emtk_core::Instance::with_path(path)?.build().await })
						.map(|result| {
							result
								.map(|instance| Message::Init(instance))
								.map_err(|e| error!("{}", e))
								.unwrap_or(Message::InitFailed)
						}),
				)
				.chain(Task::done(Message::Loaded)),
		)
	}

	// TODO: refactor into sync function that returns (Self, Task<Message>)
	#[instrument(level = "trace")]
	pub async fn with_instance(inner: emtk_core::Instance) -> Self {
		let table = Table::new(&inner);
		let mut instance = Self {
			inner,
			is_plugin_maximized: false,
			markdown_kind: None,
			plugin: Plugin::default(),
			profiles: Vec::new(),
			profile_form_input: String::new(),
			split: 0.5,
			state: State::default(),
			table,
		};
		instance.refresh_profiles().await;
		instance
	}

	#[instrument(level = "trace")]
	pub fn update(&mut self, message: Message) -> Action {
		match message {
			Message::ClickedRow(index) => {
				self.table.focus_row = Some(index);
				return Action::Task(Task::done(Message::PluginReadMarkdown(
					MarkdownKind::default(),
				)));
			}
			Message::DraggedRow(point, _bounds) => {
				if let Some(focus) = self.table.focus_row
					&& let Some(row) = self.table.rows.get(focus)
				{
					let options = self
						.table
						.rows
						.iter()
						.filter_map(|option| {
							if row.widget_id != option.widget_id {
								Some(option.widget_id.clone())
							} else {
								None
							}
						})
						.collect();
					return Action::Task(zones_on_point(
						Message::OverRow,
						point,
						Some(options),
						None,
					));
				}
			}
			Message::DraggedRowCanceled => self.table.over = None,
			Message::DroppedRow(point, _bounds) => {
				self.table.over = None;
				if let Some(focus) = self.table.focus_row
					&& let Some(row) = self.table.rows.get(focus)
				{
					let options = self
						.table
						.rows
						.iter()
						.filter_map(|option| {
							if row.widget_id != option.widget_id {
								Some(option.widget_id.clone())
							} else {
								None
							}
						})
						.collect();
					return Action::Task(zones_on_point(
						Message::ReorderRows,
						point,
						Some(options),
						None,
					));
				}
			}
			Message::EntryToggled(row_index, enabled) => {
				let load_order = self.inner.profile.load_order_mut();
				if let Some(row) = self.table.rows.get_mut(row_index)
					&& let Some(plugin) = load_order.get_mut(&row.plugin_id)
				{
					row.plugin.enabled = enabled;
					plugin.enabled = enabled;
				} else {
					error!(
						"{}",
						Error::new(
							anyhow!("index {} does not exist in load order", row_index),
							"failed to toggle mod",
						)
					);
				}

				let Ok(buffer) = toml::to_string(&load_order)
					.map_err(TomlError::from)
					.map_err(Error::msg(
						"failed to serialize profile's load order into buffer",
					))
				else {
					return Action::None;
				};
				info!("profile's load order serialized to buffer");

				let path = self.inner.profile.path().clone();
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								fs::write(path.join(emtk_core::Profile::LOAD_ORDER_TOML), buffer)
									.await
									.map_err(Error::msg(
										"failed to write profile's load order buffer into file",
									))
							})
							.map(|result| {
								result
									.map(|_| info!("finished writing profile's load order to file"))
									.map_err(|e| error!("{}", e))
							})
							.discard(),
						)
						.chain(Task::done(Message::Loaded)),
				);
			}
			Message::Init(instance) => {
				self.table = Table::new(&instance);
				self.inner = instance;
				return Action::Task(Task::done(Message::RefreshProfiles));
			}
			Message::InitFailed => return Action::InitFailed,
			Message::Launch => {
				let profile = self.inner.profile().clone();
				// TODO: env should be set within the launch() function to prevent forgetting to set this env
				unsafe {
					env::set_var(
						"EMTK_LOAD_ORDER_PATH",
						self.inner()
							.profile()
							.path()
							.join(emtk_core::Profile::LOAD_ORDER_TOML),
					);
				}
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::perform(
								async move { profile.game_dir().await.map_err(|e| error!("{}", e)) },
								|result| {
									if let Ok(path) = result {
										let _ = crate::launch(&path).map_err(|e| error!("{}", e));
									}
								},
							)
							.discard(),
						)
						.chain(Task::done(Message::Loaded)),
				);
			}
			Message::LinkClicked(url) => {
				return Action::Task(
					Task::future(async move {
						let _ = open::that_in_background(url.as_str())
							.join()
							.map(|r| {
								r.map_err(Error::msg("failed to open url"))
									.map_err(|e| error!("{}", e))
							})
							.map_err(|_| error!("failed to open url in background"));
					})
					.discard(),
				);
			}
			Message::Loaded => return Action::Loaded,
			Message::Loading => return Action::Loading,
			Message::NewPlugin => {
				error!("new plugin button work in progress")
			}
			Message::OverRow(zones) => {
				let Some((widget_id, _)) = zones.into_iter().next() else {
					return Action::None;
				};
				self.table.over = Some(widget_id);
			}
			Message::PluginClosed => {
				self.plugin = Plugin::None;
				self.markdown_kind = None;
				self.is_plugin_maximized = false;
			}
			Message::PluginDirectoryOpened(path) => {
				return Action::Task(
					Task::future(async move {
						let _ = open::that_in_background(path)
							.join()
							.map(|r| {
								r.map_err(Error::msg("failed to open plugin directory"))
									.map_err(|e| error!("{}", e))
							})
							.map_err(|_| error!("failed to open directory in background"));
					})
					.discard(),
				);
			}
			Message::PluginEditMarkdown => {
				if let Plugin::Editor { .. } = &self.plugin {
					return Action::None;
				}
				let Some(kind) = self.markdown_kind.clone() else {
					return Action::None;
				};

				let instance_path = self.inner.path().clone();
				return Action::Task(self.plugin_task(move |Row { plugin_id, .. }| {
					let plugin_id = plugin_id.clone();
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								let file_path = match kind {
									MarkdownKind::Changelog => plugin_id.changelog_file(),
									MarkdownKind::License => plugin_id.license_file(),
									MarkdownKind::Readme => plugin_id.readme_file(),
								};
								let md_path = instance_path.join(file_path);
								if md_path.is_file() {
									fs::read_to_string(md_path).await
								} else {
									Ok(String::new())
								}
							})
							.map(|result| {
								result
									.map_err(Error::msg(
										"failed to read file into buffer for text editor",
									))
									.map_err(|e| error!("{}", e))
							})
							.and_then(|buffer| Task::done(Message::TextEditorLoaded(buffer))),
						)
						.chain(Task::done(Message::Loaded))
				}));
			}
			Message::PluginMaximizeToggled => self.is_plugin_maximized = !self.is_plugin_maximized,
			Message::PluginMarkdownChanged((kind, buffer)) => {
				self.plugin = Plugin::Markdown(markdown::parse(&buffer).collect());
				self.markdown_kind = Some(kind);
			}
			Message::PluginReadMarkdown(kind) => {
				let instance_path = self.inner.path().clone();
				return Action::Task(self.plugin_task(move |Row { plugin_id, .. }| {
					let plugin_id = plugin_id.clone();
					let file_path = match kind {
						MarkdownKind::Changelog => plugin_id.changelog_file(),
						MarkdownKind::License => plugin_id.license_file(),
						MarkdownKind::Readme => plugin_id.readme_file(),
					};
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								let md_path = instance_path.join(file_path);
								(
									kind.clone(),
									if md_path.is_file() {
										fs::read_to_string(md_path)
											.await
											.map_err(Error::msg(format!(
												"failed to read {}'s {} file",
												plugin_id, kind
											)))
											.map_err(|e| error!("{}", e))
											.unwrap_or_default()
									} else {
										String::new()
									},
								)
							})
							.map(Message::PluginMarkdownChanged),
						)
						.chain(Task::done(Message::Loaded))
				}));
			}
			Message::PluginReadSettings => {
				let instance_path = self.inner.path().clone();
				return Action::Task(self.plugin_task(move |Row { plugin_id, .. }| {
					let plugin_id = plugin_id.clone();
					let file_path = instance_path.join(plugin_id.settings_file());
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								if file_path.is_file() {
									let buffer = fs::read_to_string(file_path)
										.await
										.map_err(Error::msg(format!(
											"failed to read {}'s settings file",
											plugin_id
										)))
										.map_err(|e| error!("{}", e))
										.unwrap_or_default();
									toml::from_str::<plugin::Settings>(&buffer)
										.map_err(TomlError::from)
										.map_err(Error::msg(format!(
											"failed to deserialize {}'s settings from buffer",
											plugin_id,
										)))
										.map_err(|e| error!("{}", e))
										.ok()
								} else {
									None
								}
							})
							.map(Message::PluginSettingsLoaded),
						)
						.chain(Task::done(Message::Loaded))
				}));
			}
			Message::PluginSaveMarkdown => {
				if let Plugin::Editor { content, .. } = &self.plugin
					&& let Some(kind) = &self.markdown_kind
				{
					let kind = kind.clone();
					let instance_path = self.inner.path().clone();
					let buffer = content.text();
					return Action::Task(self.plugin_task(move |Row { plugin_id, .. }| {
						let plugin_id = plugin_id.clone();
						let file_path = match kind {
							MarkdownKind::Changelog => plugin_id.changelog_file(),
							MarkdownKind::License => plugin_id.license_file(),
							MarkdownKind::Readme => plugin_id.readme_file(),
						};
						Task::done(Message::Loading)
							.chain(
								Task::future(async move {
									fs::write(instance_path.join(file_path), &buffer)
										.await
										.map_err(Error::msg(format!(
											"failed to write to {}'s {} file",
											plugin_id, kind
										)))
										.map_err(|e| error!("{}", e))?;
									Ok::<_, ()>((kind, buffer))
								})
								.and_then(|(kind, buffer)| {
									Task::done(Message::PluginMarkdownChanged((kind, buffer)))
								}),
							)
							.chain(Task::done(Message::Loaded))
					}));
				}
			}
			Message::PluginSettingsChanged(value) => {}
			Message::PluginSettingsLoaded(value) => {
				self.plugin = Plugin::Settings(value);
				self.markdown_kind = None;
			}
			Message::ProfileChanged(profile) => {
				self.inner.profile = profile;
				self.table = Table::new(&self.inner);
				let Ok(buffer) = ron::ser::to_string(self.inner.profile.path())
					.map_err(Error::msg(
						"failed to serialize path to profile into buffer",
					))
					.map_err(|e| error!("{}", e))
				else {
					return Action::None;
				};
				info!("profile path serialized into buffer");

				let path = self.inner.path().clone();
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								// TODO: clean up redundancy from `emtk_core::Instance::cache_dir`.
								let cache_dir = path
									.join(emtk_core::Instance::DATA_DIR)
									.join(emtk_core::Instance::CACHE_DIR);
								emtk_core::ensure_dir(&cache_dir).await?;

								fs::write(
									cache_dir.join(emtk_core::Instance::RECENT_PROFILE_RON),
									buffer,
								)
								.await
							})
							.map(|result| {
								result
									.map(|_| info!("profile path cached to file"))
									.map_err(|e| error!("{}", e))
							})
							.discard(),
						)
						.chain(Task::done(Message::Loaded)),
				);
			}
			Message::ProfileDeleted => {
				let path = self.inner.profile().path().clone();
				return Action::Task(
					Task::done(Message::Loading).chain(
						Task::future(async move {
							fs::remove_dir_all(path)
								.await
								.map_err(Error::msg("failed to delete profile directory"))
								.map_err(|e| error!("{}", e))
						})
						.and_then(|_| {
							Task::batch([
								Task::done(Message::ProfileSelected(
									emtk_core::Instance::DEFAULT_PROFILE_DIR.to_string(),
								)),
								Task::done(Message::RefreshProfiles),
							])
						}),
					),
				);
			}
			Message::ProfileFormInputChanged(input) => self.profile_form_input = input,
			Message::ProfileFormSubmitted => {
				let path = self.inner.path().clone();
				let profile_form_input = self.profile_form_input.clone();
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								// TODO: clean up redundancy from `emtk_core::Instance::profile_dirs`.
								let profiles_dir = path
									.join(emtk_core::Instance::DATA_DIR)
									.join(emtk_core::Instance::PROFILES_DIR);
								emtk_core::ensure_dir(&profiles_dir).await.map_err(Error::msg(
									"failed to ensure profiles directory exists",
								))?;

								let new_dir = profiles_dir.join(&profile_form_input);
								if new_dir.is_dir() {
									return Err(Error::new(
										anyhow!("directory already exists"),
										"failed to create profile with name",
									));
								}
								emtk_core::Profile::with_path(new_dir).await?.build().await
							})
							.map(|result| result.map_err(|e| error!("{}", e)))
							.and_then(|profile| {
								Task::batch([
									Task::done(Message::ProfileChanged(profile)),
									Task::done(Message::RefreshProfiles),
									Task::done(Message::StateChanged(State::Default)),
								])
							}),
						)
						.chain(Task::done(Message::Loaded)),
				);
			}
			Message::Profiles(profile_paths) => self.profiles = profile_paths,
			Message::ProfileSelected(name) => {
				let path = self.inner.path().clone();
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								// TODO: clean up redundancy from `emtk_core::Instance::profile_dirs`.
								let profiles_dir = path
									.join(emtk_core::Instance::DATA_DIR)
									.join(emtk_core::Instance::PROFILES_DIR);
								emtk_core::ensure_dir(&profiles_dir).await.map_err(Error::msg(
									"failed to ensure profiles directory exists",
								))?;

								emtk_core::Profile::with_path(profiles_dir.join(name))
									.await?
									.build()
									.await
							})
							.map(|result| result.map_err(|e| error!("{}", e)))
							.and_then(|profile| Task::done(Message::ProfileChanged(profile))),
						)
						.chain(Task::done(Message::Loaded)),
				);
			}
			Message::RefreshProfiles => {
				let path = self.inner.path().clone();
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								// TODO: clean up redundancy from `emtk_core::Instance::profile_dirs`.
								let profiles_dir = path
									.join(emtk_core::Instance::DATA_DIR)
									.join(emtk_core::Instance::PROFILES_DIR);
								emtk_core::ensure_dir(&profiles_dir).await?;

								let mut profile_paths = Vec::new();
								let mut read_profiles_dir = fs::read_dir(&profiles_dir).await?;
								while let Some(entry) = read_profiles_dir.next_entry().await? {
									let entry_path = entry.path();
									if entry_path.is_dir() {
										profile_paths.push(entry_path);
									};
								}
								Ok(profile_paths)
							})
							.map(|result| {
								result
									.map_err(Error::msg::<io::Error, _>(
										"failed to refresh profiles",
									))
									.map_err(|e| error!("{}", e))
							})
							.and_then(|profile_paths| Task::done(Message::Profiles(profile_paths))),
						)
						.chain(Task::done(Message::Loaded)),
				);
			}
			Message::ReorderRows(zones) => {
				let Some((widget_id, _)) = zones.first() else {
					return Action::None;
				};

				let Some(row_index) = self.table.focus_row else {
					return Action::None;
				};

				let Some(to_index) = self.table.rows.iter().enumerate().find_map(|(i, row)| {
					if row.widget_id == *widget_id {
						Some(i)
					} else {
						None
					}
				}) else {
					return Action::None;
				};

				self.inner
					.profile_mut()
					.load_order_mut()
					.iter_mut()
					.for_each(move |(_, plugin)| {
						if plugin.priority == row_index as u32 {
							plugin.priority = to_index as u32;
						} else if row_index < to_index
							&& plugin.priority >= row_index as u32
							&& plugin.priority <= to_index as u32
							&& plugin.priority != 0
						{
							plugin.priority -= 1;
						} else if row_index > to_index
							&& plugin.priority <= row_index as u32
							&& plugin.priority >= to_index as u32
						{
							plugin.priority += 1;
						}
					});
				self.table = Table::new(&self.inner);
				let path = self.inner.profile.path().clone();
				let Ok(buffer) = toml::to_string(self.inner.profile().load_order())
					.map_err(TomlError::from)
					.map_err(Error::msg(
						"failed to serialize profile's load order into buffer",
					))
					.map_err(|e| error!("{}", e))
				else {
					return Action::None;
				};
				info!("profile's load order serialized to buffer");
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								fs::write(path.join(emtk_core::Profile::LOAD_ORDER_TOML), buffer).await
							})
							.map(|result| {
								result
									.map(|_| info!("finished writing profile's load order to file"))
									.map_err(|e| error!("{}", e))
							})
							.discard(),
						)
						.chain(Task::done(Message::Loaded)),
				);
			}
			Message::SettingsPressed => {
				// TODO: implement instance settings
				error!("instance settings button work in progress")
			}
			Message::SplitDragged(split) => self.split = split,
			Message::StateChanged(state) => {
				if self.state == State::ProfileForm {
					self.profile_form_input = String::new()
				}
				self.state = state;
			}
			Message::TableResized => self.table.columns.iter_mut().for_each(|column| {
				if let Some(offset) = column.resize_offset.take() {
					column.width += offset;
				}
			}),
			Message::TableResizing(index, offset) => {
				if let Some(column) = self.table.columns.get_mut(index) {
					column.resize_offset = Some(offset);
				}
			}
			Message::TableSyncHeader(offset) => {
				return Action::Task(scrollable::scroll_to(self.table.header.clone(), offset));
			}
			Message::TextEditorLoaded(buffer) => {
				let mut content = text_editor::Content::new();
				content.perform(text_editor::Action::Edit(text_editor::Edit::Paste(
					buffer.into(),
				)));
				let preview = markdown::parse(&content.text()).collect();
				self.plugin = Plugin::Editor {
					content,
					is_dirty: false,
					is_preview: false,
					preview,
				};
			}
			Message::TextEditorAction(action) => {
				if let Plugin::Editor {
					content, is_dirty, ..
				} = &mut self.plugin
				{
					if !*is_dirty && action.is_edit() {
						*is_dirty = true;
					}
					content.perform(action);
				}
			}
		}

		Action::None
	}

	#[instrument(level = "trace")]
	pub fn view<'a>(&'a self, root: &'a Root) -> Element<'a, Message> {
		let profile_controls: Element<_> = if self.profiles.is_empty() {
			tooltip(
				text("Error").style(text::danger),
				text("failed to detect any profiles"),
				tooltip::Position::Bottom,
			)
			.into()
		} else if self.state == State::ProfileForm {
			self.profile_form()
		} else {
			self.profile_picker()
		};

		let instance_settings_btn = tooltip(
			button(icon::settings().size(16).center())
				.on_press(Message::SettingsPressed)
				.width(31)
				.height(31),
			text("Instance settings"),
			tooltip::Position::Bottom,
		);

		let play_btn = button(center_x(
			row![
				icon::play().size(18).center(),
				text("Play").size(16).center()
			]
			.align_y(Alignment::Center)
			.spacing(4),
		))
		.on_press(Message::Launch)
		.width(120)
		.height(31)
		.style(|theme, status| button::Style {
			border: Border::default().rounded(3),
			..iced::widget::button::success(theme, status)
		});

		let controls = row![
			profile_controls,
			horizontal_space(),
			instance_settings_btn,
			play_btn
		]
		.align_y(Alignment::Center)
		.spacing(4)
		.padding([0, 8]);

		let table = responsive(|size| {
			table(
				self.table.header.clone(),
				self.table.body.clone(),
				&self.table.columns,
				&self.table.rows,
				Message::TableSyncHeader,
			)
			.on_column_resize(Message::TableResizing, Message::TableResized)
			.on_row_click(Message::ClickedRow)
			.on_row_drop(Message::DroppedRow)
			.min_width(size.width)
			.into()
		});

		let content = column![controls, table].spacing(8).padding([8, 0]);

		let plugin_content = match &self.plugin {
			Plugin::Editor { content, .. } => self.plugin_editor(content),
			Plugin::Markdown(md) => self.plugin_markdown(root, md),
			Plugin::None => return content.into(),
			Plugin::Settings(value) => self.plugin_settings(),
		};

		if self.is_plugin_maximized {
			plugin_content
		} else {
			Split::new(content, plugin_content, self.split, Message::SplitDragged)
				.direction(Direction::Horizontal)
				.style(|theme| rule::Style {
					width: 0,
					..rule::default(theme)
				})
				.into()
		}
	}

	#[instrument(level = "trace")]
	pub fn title(&self) -> String {
		self.inner
			.settings()
			.name
			.clone()
			.unwrap_or(self.inner.path().display().to_string())
	}

	#[instrument(level = "trace")]
	fn plugin_controls(&self) -> Element<'_, Message> {
		use MarkdownKind::*;
		container(
			row![
				self.md_tab_btn(icon::book_open(), "README", Readme)
					.on_press(Message::PluginReadMarkdown(Readme)),
				self.md_tab_btn(icon::scroll_text(), "Changelog", Changelog)
					.on_press(Message::PluginReadMarkdown(Changelog)),
				self.md_tab_btn(icon::scale(), "License", License)
					.on_press(Message::PluginReadMarkdown(License)),
				self.settings_tab_btn(if let Plugin::Settings(_) = self.plugin {
					true
				} else {
					false
				}),
				horizontal_space(),
				tooltip(
					button(icon::pen().center())
						.on_press(Message::PluginEditMarkdown)
						.width(31)
						.height(31),
					"Edit",
					tooltip::Position::Bottom
				),
				vertical_rule(1),
				button(
					if self.is_plugin_maximized {
						icon::minimize()
					} else {
						icon::maximize()
					}
					.size(18)
					.center()
				)
				.on_press(Message::PluginMaximizeToggled)
				.width(31)
				.height(31),
				button(icon::close().size(16).center())
					.on_press(Message::PluginClosed)
					.width(31)
					.height(31)
					.style(button::danger),
			]
			.align_y(Alignment::Center)
			.spacing(8)
			.padding(8),
		)
		.height(44)
		.into()
	}

	#[instrument(level = "trace")]
	fn plugin_editor<'a>(&'a self, content: &'a text_editor::Content) -> Element<'a, Message> {
		responsive(move |size| {
			center_x(
				container(
					column![
						self.plugin_controls(),
						horizontal_rule(1),
						text_editor(content)
							.on_action(Message::TextEditorAction)
							.height(Fill)
					]
					.width(if size.width > MD_WIDTH {
						Length::Fixed(MD_WIDTH)
					} else {
						Fill
					}),
				)
				.style(move |theme: &Theme| {
					if size.width > MD_WIDTH {
						container::Style {
							background: Some(theme.palette().background.into()),
							shadow: Shadow::default(),
							border: Border {
								color: theme.extended_palette().background.strong.color,
								width: 1.,
								..Border::default()
							},
							..container::bordered_box(theme)
						}
					} else {
						container::Style::default()
					}
				}),
			)
			.style(move |theme: &Theme| {
				let default = container::Style::default();
				if size.width > MD_WIDTH {
					container::Style {
						background: Some(
							theme
								.extended_palette()
								.secondary
								.weak
								.color
								.scale_alpha(0.2)
								.into(),
						),
						..default
					}
				} else {
					default
				}
			})
			.into()
		})
		.into()
	}

	#[instrument(level = "trace")]
	fn plugin_markdown<'a>(
		&'a self,
		root: &'a Root,
		md: &'a Vec<markdown::Item>,
	) -> Element<'a, Message> {
		responsive(move |size| {
			let content = center_x(
				container(
					column![
						self.plugin_controls(),
						horizontal_rule(1),
						scrollable(
							container(markdown(md, root.theme.clone()).map(Message::LinkClicked),)
								.padding(32),
						)
						.width(Fill)
						.height(Fill)
					]
					.width(if size.width > MD_WIDTH {
						Length::Fixed(MD_WIDTH)
					} else {
						Fill
					}),
				)
				.style(move |theme: &Theme| {
					if size.width > MD_WIDTH {
						container::Style {
							background: Some(theme.palette().background.into()),
							shadow: Shadow::default(),
							border: Border {
								color: theme.extended_palette().background.strong.color,
								width: 1.,
								..Border::default()
							},
							..container::bordered_box(theme)
						}
					} else {
						container::Style::default()
					}
				}),
			)
			.style(move |theme: &Theme| {
				let default = container::Style::default();
				if size.width > MD_WIDTH {
					container::Style {
						background: Some(
							theme
								.extended_palette()
								.secondary
								.weak
								.color
								.scale_alpha(0.2)
								.into(),
						),
						..default
					}
				} else {
					default
				}
			});

			let content: Element<_> = if size.width > MD_WIDTH {
				content.into()
			} else {
				column![horizontal_rule(1), content].into()
			};

			content.into()
		})
		.into()
	}

	#[instrument(level = "trace")]
	fn plugin_settings(&self) -> Element<'_, Message> {
		let settings: Element<_> = if let Plugin::Settings(maybe_settings) = &self.plugin
			&& let Some(settings) = maybe_settings
		{
			dbg!(&settings);
			Space::new(Fill, Fill).into()
			// self.view_toml(settings.clone())
		} else {
			Space::new(Fill, Fill).into()
		};

		let info: Element<_> = if let Some(row_index) = self.table.focus_row
			&& let Some(Row { plugin_id, .. }) = self.table.rows.get(row_index)
		{
			button(
				row![
					icon::square_arrow_out_up_right().center(),
					text("Open plugin directory")
				]
				.align_y(Alignment::Center)
				.spacing(6),
			)
			.on_press(Message::PluginDirectoryOpened(
				self.inner.path().join(plugin_id.plugin_dir()),
			))
			.into()
		} else {
			Space::new(Fill, Fill).into()
		};

		column![
			horizontal_rule(1),
			self.plugin_controls(),
			horizontal_rule(1),
			container(settings).width(Fill).height(Fill),
			horizontal_rule(1),
			container(info).width(Fill).height(Fill)
		]
		.into()
	}

	#[instrument(level = "trace")]
	pub fn profile_form<'a>(&self) -> Element<'a, Message> {
		row![
			text_input("New Profile...", &self.profile_form_input)
				.on_input(Message::ProfileFormInputChanged)
				.on_submit_maybe(if self.profile_form_input.is_empty() {
					None
				} else {
					Some(Message::ProfileFormSubmitted)
				})
				.width(160),
			tooltip(
				button(icon::check().size(12).center())
					.on_press_maybe(if self.profile_form_input.is_empty() {
						None
					} else {
						Some(Message::ProfileFormSubmitted)
					})
					.width(31)
					.height(31)
					.style(button::success),
				text("Create"),
				tooltip::Position::Bottom,
			),
			tooltip(
				button(icon::close().size(12).center())
					.on_press(Message::StateChanged(State::Default))
					.width(31)
					.height(31)
					.style(button::danger),
				text("Cancel"),
				tooltip::Position::Bottom
			),
		]
		.into()
	}

	#[instrument(level = "trace")]
	pub fn profile_picker<'a>(&self) -> Element<'a, Message> {
		row![
			tooltip(
				pick_list(
					self.profiles
						.iter()
						.filter_map(|path| path.file_name().map(|name| name.display().to_string()))
						.collect::<Vec<_>>(),
					self.inner
						.profile()
						.path()
						.file_name()
						.map(|name| name.display().to_string()),
					Message::ProfileSelected
				)
				.width(160),
				text("Current profile"),
				tooltip::Position::Top
			),
			tooltip(
				button(icon::plus().size(12).center())
					.on_press(Message::StateChanged(State::ProfileForm))
					.width(31)
					.height(31),
				text("New profile"),
				tooltip::Position::Bottom
			),
			tooltip(
				button(icon::trash().size(16).center())
					.on_press(Message::ProfileDeleted)
					.width(31)
					.height(31)
					.style(button::danger),
				text("Delete profile"),
				tooltip::Position::Bottom
			)
		]
		.into()
	}

	fn md_tab_btn<'a>(
		&'a self,
		icon: Text<'a>,
		name: &'a str,
		kind: MarkdownKind,
	) -> Button<'a, Message> {
		button(
			row![icon.center(), text(name).center()]
				.align_y(Alignment::Center)
				.spacing(6),
		)
		.style(
			move |theme: &Theme, status: button::Status| -> button::Style {
				let ext_palette = theme.extended_palette();
				let primary = button::primary(theme, status);
				if status != button::Status::Hovered
					&& let Some(md_kind) = &self.markdown_kind
					&& *md_kind == kind
				{
					button::Style {
						background: Some(ext_palette.background.strong.color.into()),
						text_color: ext_palette.background.strong.text,
						..primary
					}
				} else {
					primary
				}
			},
		)
	}

	fn settings_tab_btn<'a>(&'a self, is_settings: bool) -> Button<'a, Message> {
		button(
			row![icon::settings().center(), text("Settings")]
				.align_y(Alignment::Center)
				.spacing(6),
		)
		.on_press(Message::PluginReadSettings)
		.style(move |theme: &Theme, status: button::Status| {
			let ext_palette = theme.extended_palette();
			let primary = button::primary(theme, status);
			if status != button::Status::Hovered && is_settings {
				button::Style {
					background: Some(ext_palette.background.strong.color.into()),
					text_color: ext_palette.background.strong.text,
					..primary
				}
			} else {
				primary
			}
		})
	}

	/// Helper to create a task on the currently focused plugin.
	fn plugin_task(&self, f: impl FnOnce(&Row) -> Task<Message>) -> Task<Message> {
		let rows = &self.table.rows;
		self.table
			.focus_row
			.map(move |i| rows.get(i))
			.flatten()
			.map(move |row| f(row))
			.unwrap_or(Task::none())
	}

	/// Attempts to iterate the filesystem for valid directories of profiles related
	/// to the current instance. If an error occurs, it will be logged and not
	/// mutate self.
	#[instrument(level = "trace")]
	pub async fn refresh_profiles(&mut self) -> &mut Self {
		self.profiles = match self.inner.profile_dirs().await {
			Ok(profile_dirs) => profile_dirs,
			Err(e) => {
				error!("{}", e);
				return self;
			}
		};
		self
	}
}

impl Default for Instance {
	fn default() -> Self {
		Self {
			inner: emtk_core::Instance::default(),
			is_plugin_maximized: false,
			markdown_kind: None,
			plugin: Plugin::default(),
			profiles: Vec::new(),
			profile_form_input: String::new(),
			split: 0.5,
			state: State::default(),
			table: Table::default(),
		}
	}
}

pub fn view_checkbox<'a>(plugin_checkbox: plugin::Checkbox) -> Element<'a, Message> {
	Space::new(Fill, Fill).into()
}

pub fn view_dropdown<'a>(plugin_dropdown: plugin::Dropdown) -> Element<'a, Message> {
	Space::new(Fill, Fill).into()
}

pub fn view_radio<'a>(plugin_radio: plugin::Radio) -> Element<'a, Message> {
	Space::new(Fill, Fill).into()
}

pub fn view_slider<'a>(plugin_slider: plugin::Slider) -> Element<'a, Message> {
	Space::new(Fill, Fill).into()
}

pub fn view_text_input<'a>(plugin_input: plugin::TextInput) -> Element<'a, Message> {
	Space::new(Fill, Fill).into()
}
