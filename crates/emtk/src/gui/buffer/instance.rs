use std::{
	env,
	path::{Path, PathBuf},
};

use emcore::{instance, plugin, profile};
use getset::Getters;
use iced::{
	advanced::widget as iced_widget,
	widget::{
		center_x, checkbox, column, container, horizontal_space, pick_list, responsive, row, text,
		text_input,
	},
	Alignment, Border, Element, Fill, Font, Point, Rectangle, Renderer, Task, Theme,
};
use iced_drop::zones_on_point;
use iced_table::table;
use itertools::Itertools;
use tokio::fs;
use tracing::{error, info};

use crate::gui::widget::{button, icon, scrollable, tooltip};

pub mod error {
	#[derive(Debug, thiserror::Error)]
	pub enum Instance {
		#[error("{0}")]
		RonFile(#[from] emcore::error::RonFile),
		#[error("{0}")]
		Builder(#[from] emcore::instance::error::Builder),
		#[error("{0}")]
		Build(#[from] emcore::instance::error::Build),
		#[error("{0}")]
		HistoryEmpty(&'static str),
	}
}

pub enum Action {
	None,
	Loaded,
	Loading,
	Task(Task<Message>),
}

#[derive(Debug, Clone)]
pub enum ColumnKind {
	Name,
	Version,
	Priority,
}

#[derive(Debug, Clone)]
pub struct Column {
	kind: ColumnKind,
	width: f32,
	resize_offset: Option<f32>,
}

impl Column {
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

	fn header(&'a self, _col_index: usize) -> Element<'a, Message> {
		let content = match self.kind {
			ColumnKind::Name => "Name",
			ColumnKind::Version => "Version",
			ColumnKind::Priority => "Priority",
		};

		text(content).into()
	}

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

	fn width(&self) -> f32 {
		self.width
	}

	fn resize_offset(&self) -> Option<f32> {
		self.resize_offset
	}
}

#[derive(Debug, Clone)]
pub struct Row {
	widget_id: iced_widget::Id,
	plugin_id: plugin::Id,
	plugin: profile::LoadOrderEntry,
}

impl iced_table::WithId for Row {
	fn id(&self) -> iced::advanced::graphics::core::widget::Id {
		self.widget_id.clone()
	}
}

#[derive(Debug, Clone)]
pub struct Table {
	body: scrollable::Id,
	columns: Vec<Column>,
	focus_row: Option<usize>,
	footer: scrollable::Id,
	header: scrollable::Id,
	over: Option<iced_widget::Id>,
	rows: Vec<Row>,
}

impl Table {
	pub fn new(instance: &emcore::Instance) -> Self {
		let mut table = Self {
			body: scrollable::Id::unique(),
			columns: vec![
				Column::new(ColumnKind::Name),
				Column::new(ColumnKind::Version),
				Column::new(ColumnKind::Priority),
			],
			focus_row: None,
			footer: scrollable::Id::unique(),
			header: scrollable::Id::unique(),
			over: None,
			rows: Vec::new(),
		};
		table.refresh(instance.profile().load_order().clone());
		table
	}

	/// Fills the table's rows with the current profile's load order. This can be
	/// used in combination with `Instance::refresh` to fully update the load order.
	pub fn refresh(&mut self, load_order: profile::LoadOrder) -> &mut Self {
		self.rows = load_order
			.into_iter()
			.sorted_by(|(_, a), (_, b)| Ord::cmp(&a.priority, &b.priority))
			.map(|(plugin_id, plugin)| Row {
				widget_id: iced_widget::Id::unique(),
				plugin_id,
				plugin,
			})
			.collect();
		self
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum State {
	#[default]
	Default,
	ProfileForm,
}

#[derive(Debug, Clone, Getters)]
pub struct Instance {
	#[getset(get = "pub")]
	inner: emcore::Instance,
	profiles: Vec<PathBuf>,
	profile_form_input: String,
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
	Launch,
	Loaded,
	Loading,
	NewPlugin,
	OverRow(Vec<(iced_widget::Id, Rectangle)>),
	ProfileDeleted,
	ProfileFormInputChanged(String),
	ProfileFormSubmitted,
	ProfileSelected(String),
	Refresh(Box<Instance>),
	ReorderRows(Vec<(iced_widget::Id, Rectangle)>),
	SettingsPressed,
	StateChanged(State),
	TableResized,
	TableResizing(usize, f32),
	TableSyncHeader(scrollable::AbsoluteOffset),
}

impl Instance {
	pub async fn new() -> Result<Self, error::Instance> {
		let instance_history = instance::history().await?;
		let Some(instance_path) = instance_history.last() else {
			return Err(error::Instance::HistoryEmpty(
				"failed to find last used instance in history",
			));
		};

		let inner = emcore::Instance::with_path(instance_path)?.build().await?;

		let table = Table::new(&inner);

		let mut instance = Self {
			inner,
			profiles: Vec::new(),
			profile_form_input: String::new(),
			state: State::default(),
			table,
		};
		instance.refresh_profiles().await;
		Ok(instance)
	}

	pub async fn with_path(path: &Path) -> Result<Self, error::Instance> {
		let inner = emcore::Instance::with_path(path)?.build().await?;
		let table = Table::new(&inner);
		let mut instance = Self {
			inner,
			profiles: Vec::new(),
			profile_form_input: String::new(),
			state: State::default(),
			table,
		};
		instance.refresh_profiles().await;
		Ok(instance)
	}

	pub async fn with_instance(inner: emcore::Instance) -> Self {
		let table = Table::new(&inner);
		let mut instance = Self {
			inner,
			profiles: Vec::new(),
			profile_form_input: String::new(),
			state: State::default(),
			table,
		};
		instance.refresh_profiles().await;
		instance
	}

	pub fn update(&mut self, message: Message) -> Action {
		match message {
			Message::ClickedRow(index) => self.table.focus_row = Some(index),
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
				let mut load_order = self.inner.profile().load_order().clone();
				if let Some(row) = self.table.rows.get_mut(row_index)
					&& let Some(plugin) = load_order.get_mut(&row.plugin_id)
				{
					row.plugin.enabled = enabled;
					plugin.enabled = enabled;
					info!(
						"{} set to {}",
						plugin
							.display_name
							.clone()
							.unwrap_or(row.plugin_id.to_string()),
						plugin.enabled
					)
				} else {
					error!(
						"failed to toggle entry #{} to {}, doing nothing",
						row_index, enabled
					);
					return Action::None;
				}

				let mut instance = self.inner.clone();
				return Action::Task(
					Task::future(async move {
						match instance.profile_mut().set_load_order(load_order).await {
							Ok(_) => Ok(Instance::with_instance(instance).await),
							Err(e) => {
								error!("{}", e);
								Err(e)
							}
						}
					})
					.and_then(|instance| Task::done(Message::Refresh(Box::new(instance))))
					.chain(Task::done(Message::Loaded)),
				);
			}
			Message::Launch => {
				let instance_path = self.inner().path().clone();
				let load_order = self.inner().profile().load_order().clone();
				let profile = self.inner.profile().clone();
				// TODO: env should be set within the launch() function to prevent forgetting to set this env
				env::set_var(
					"EMTK_LOAD_ORDER_PATH",
					self.inner()
						.profile()
						.path()
						.join(emcore::Profile::LOAD_ORDER_TOML),
				);
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
			Message::ProfileDeleted => {
				let path = self.inner.profile().path().clone();
				return Action::Task(
					Task::done(Message::Loading).chain(
						Task::future(async move {
							fs::remove_dir_all(path)
								.await
								.map_err(|source| emcore::error::Io {
									message: "failed to delete profile directory",
									source,
								})
								.map_err(|e| error!("{}", e))
						})
						.and_then(|_| {
							Task::done(Message::ProfileSelected(
								emcore::Instance::DEFAULT_PROFILE_DIR.to_string(),
							))
						}),
					),
				);
			}
			Message::ProfileFormInputChanged(input) => self.profile_form_input = input,
			Message::ProfileFormSubmitted => {
				let mut instance = self.inner.clone();
				let profile_form_input = self.profile_form_input.clone();
				return Action::Task(
					Task::done(Message::Loading).chain(
						Task::future(async move {
							let profiles_dir = match instance.profiles_dir().await {
								Ok(profiles_dir) => profiles_dir,
								Err(e) => {
									error!("{}", e);
									return Err(());
								}
							};
							let new_dir = profiles_dir.join(&profile_form_input);
							if new_dir.is_dir() {
								error!("Profile with name already exists");
								return Err(());
							}
							let profile_builder = match emcore::Profile::with_path(new_dir).await {
								Ok(profile_builder) => profile_builder,
								Err(e) => {
									error!("{}", e);
									return Err(());
								}
							};
							let profile = match profile_builder.build().await {
								Ok(profile) => profile,
								Err(e) => {
									error!("{}", e);
									return Err(());
								}
							};
							if let Err(e) = instance.set_profile(profile).await {
								error!("{}", e);
							};

							Ok(Instance::with_instance(instance).await)
						})
						.and_then(|instance| Task::done(Message::Refresh(Box::new(instance))))
						.chain(Task::done(Message::Loaded)),
					),
				);
			}
			Message::ProfileSelected(name) => {
				let mut instance = self.inner.clone();

				return Action::Task(
					Task::done(Message::Loading).chain(
						Task::future(async move {
							let profiles_dir = match instance.profiles_dir().await {
								Ok(profiles_dir) => profiles_dir,
								Err(e) => {
									error!("{}", e);
									return Err(());
								}
							};
							let selected_dir = profiles_dir.join(name);
							let profile_builder =
								match emcore::Profile::with_path(selected_dir).await {
									Ok(profile_builder) => profile_builder,
									Err(e) => {
										error!("{}", e);
										return Err(());
									}
								};
							let profile = match profile_builder.build().await {
								Ok(profile) => profile,
								Err(e) => {
									error!("{}", e);
									return Err(());
								}
							};

							if let Err(e) = instance.set_profile(profile).await {
								error!("{}", e);
							};

							Ok(Instance::with_instance(instance).await)
						})
						.and_then(|instance| Task::done(Message::Refresh(Box::new(instance))))
						.chain(Task::done(Message::Loaded)),
					),
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

				let mut load_order = self.inner.profile().load_order().clone();
				for (_, plugin) in load_order.iter_mut() {
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
				}

				let mut instance = self.inner.clone();
				return Action::Task(
					Task::done(Message::Loading).chain(
						Task::future(async move {
							match instance.profile_mut().set_load_order(load_order).await {
								Ok(_) => Ok(Instance::with_instance(instance).await),
								Err(e) => {
									error!("{}", e);
									Err(e)
								}
							}
						})
						.and_then(|instance| Task::done(Message::Refresh(Box::new(instance))))
						.chain(Task::done(Message::Loaded)),
					),
				);
			}
			Message::Refresh(instance) => *self = *instance,
			Message::SettingsPressed => {
				// TODO: implement instance settings
				error!("instance settings button work in progress")
			}
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
				return Action::Task(Task::batch([
					scrollable::scroll_to(self.table.header.clone(), offset),
					scrollable::scroll_to(self.table.footer.clone(), offset),
				]))
			}
		}

		Action::None
	}

	pub fn view(&self) -> Element<Message> {
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
			// .footer(self.table.footer.clone())
			.on_column_resize(Message::TableResizing, Message::TableResized)
			.on_row_click(Message::ClickedRow)
			.on_row_drop(Message::DroppedRow)
			.min_width(size.width)
			.into()
		});

		column![controls, table].spacing(8).into()
	}

	pub fn title(&self) -> String {
		self.inner
			.settings()
			.name
			.clone()
			.unwrap_or(self.inner.path().display().to_string())
	}

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

	/// Attempts to iterate the filesystem for valid directories of profiles related
	/// to the current instance. If an error occurs, it will be logged and not
	/// mutate self.
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
