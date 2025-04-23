use std::{
	path::{Path, PathBuf},
	time::UNIX_EPOCH,
};

use emcore::{instance, plugin, profile};
use exparser::{
	deku::{reader::Reader, writer::Writer, DekuContainerRead, DekuReader, DekuWriter},
	rpk, Format, Rpk,
};
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
use tokio::{
	fs,
	io::{self, AsyncReadExt, AsyncWriteExt},
};
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
				return Action::Task(
					Task::done(Message::Loading)
						.chain(
							Task::future(async move {
								load_mods(&instance_path, load_order).await;
							})
							.discard(),
						)
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

// TODO: huge function, separate logic into independent functions
async fn load_mods(path: &Path, load_order: profile::LoadOrder) {
	let mut load_order: Vec<_> = load_order
		.into_iter()
		.filter(|(_, entry)| {
			entry.display_name.is_some() && entry.version.is_some() && entry.enabled
		})
		.collect();
	let cache_build_dir = path
		.join(emcore::Instance::DATA_DIR)
		.join(emcore::Instance::CACHE_DIR)
		.join(emcore::Instance::CACHE_BUILD_DIR);
	if load_order.is_empty() {
		// TODO: temporary solution to running vanilla exanima
		if cache_build_dir.is_dir() {
			fs::remove_dir_all(cache_build_dir).await.unwrap();
		}
		return;
	}
	load_order.sort_by(|(_, a), (_, b)| a.priority.cmp(&b.priority));
	let native_packages: Vec<_> = path
		.read_dir()
		.expect("error while reading exanima directory")
		.flatten()
		.filter_map(|entry| {
			let path = entry.path();
			let file_name = path.display().to_string();
			if path.is_dir() || !file_name.ends_with(".rpk") {
				None
			} else {
				Some(path)
			}
		})
		.collect();

	let metadata_path = cache_build_dir.join(emcore::cache::METADATA_RON);
	let cache_load_order_path = cache_build_dir.join(emcore::Profile::LOAD_ORDER_RON);
	'check_metadata: {
		// TODO: detect for any new mods added
		if metadata_path.is_file() && cache_load_order_path.is_file() {
			let Ok(file) = fs::File::open(&metadata_path).await else {
				break 'check_metadata;
			};
			let mut reader = io::BufReader::new(file);
			let mut buffer = String::new();
			if reader.read_to_string(&mut buffer).await.is_err() {
				break 'check_metadata;
			};
			let Ok(mut de) = ron::de::Deserializer::from_str(&buffer) else {
				break 'check_metadata;
			};
			let Ok(metadata) = emcore::cache::deserialize_metadata(&mut de) else {
				break 'check_metadata;
			};
			for (path, cache_time) in metadata.iter() {
				let Ok(rpk_file) = fs::File::open(path).await else {
					break 'check_metadata;
				};
				let time = rpk_file
					.metadata()
					.await
					.unwrap()
					.modified()
					.unwrap()
					.duration_since(UNIX_EPOCH)
					.unwrap()
					.as_secs();
				if *cache_time != time {
					break 'check_metadata;
				}
			}

			let Ok(file) = fs::File::open(&cache_load_order_path).await else {
				break 'check_metadata;
			};
			let mut reader = io::BufReader::new(file);
			let mut buffer = String::new();
			if reader.read_to_string(&mut buffer).await.is_err() {
				break 'check_metadata;
			}
			let Ok(cached_load_order) =
				ron::from_str::<Vec<(plugin::Id, profile::LoadOrderEntry)>>(&buffer)
			else {
				break 'check_metadata;
			};
			if load_order
				.clone()
				.into_iter()
				.map(|(id, entry)| {
					(
						id,
						profile::LoadOrderEntry::new(entry.enabled, entry.priority, None, None),
					)
				})
				.collect::<Vec<_>>()
				!= cached_load_order
			{
				break 'check_metadata;
			}

			for native_rpk_path in native_packages.iter() {
				let native_rpk_name = native_rpk_path.file_name().unwrap().display().to_string();
				for (id, _) in load_order.iter() {
					let foreign_rpk_path = path
						.join(emcore::Instance::MODS_DIR)
						.join(id.to_string())
						.join(emcore::Instance::ASSETS_DIR)
						.join(emcore::Instance::PACKAGES_DIR)
						.join(&native_rpk_name);
					if foreign_rpk_path.is_file() {
						if metadata.contains_key(&foreign_rpk_path.canonicalize().unwrap()) {
							break 'check_metadata;
						}
					} else if foreign_rpk_path.is_dir() {
						async fn is_cache_valid(
							path: &Path,
							metadata: &emcore::cache::Metadata,
						) -> bool {
							let mut read_plugin_dir = fs::read_dir(path).await.unwrap();
							while let Some(entry) = read_plugin_dir.next_entry().await.unwrap() {
								let entry_path = entry.path();
								if entry_path.is_dir() {
									return Box::pin(is_cache_valid(path, metadata)).await;
								} else if entry_path.is_file() {
									if metadata.contains_key(&entry_path.canonicalize().unwrap()) {
										return false;
									}
								}
							}

							true
						}
						if !is_cache_valid(&foreign_rpk_path, &metadata).await {
							break 'check_metadata;
						}
					}
				}
			}

			return;
		}
	}
	let mut metadata = emcore::cache::Metadata::new();

	// TODO: support rebuilding only necessary rpks rather than all files

	for native_rpk_path in native_packages {
		let native_rpk_name = native_rpk_path.file_name().unwrap().display().to_string();

		let native_rpk_file = fs::File::open(&native_rpk_path)
			.await
			.expect("error while opening exanima file");
		let time = native_rpk_file
			.metadata()
			.await
			.unwrap()
			.modified()
			.unwrap()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs();
		metadata.insert(native_rpk_path, time);

		let mut buf_reader = io::BufReader::new(native_rpk_file);
		let mut buffer = Vec::new();
		buf_reader.read_to_end(&mut buffer).await.unwrap();
		let mut cursor = std::io::Cursor::new(buffer);
		let mut reader = Reader::new(&mut cursor);

		let mut native_rpk_format = Format::from_reader_with_ctx(&mut reader, ()).unwrap();

		if let Format::Rpk(native_package) = &mut native_rpk_format {
			let mut native_entries = native_package.entries.to_vec();
			native_entries.sort_by(|a, b| a.offset.cmp(&b.offset));

			for (id, _) in load_order.iter() {
				let mut foreign_rpk_path = path
					.join(emcore::Instance::MODS_DIR)
					.join(id.to_string())
					.join(emcore::Instance::ASSETS_DIR)
					.join(emcore::Instance::PACKAGES_DIR)
					.join(&native_rpk_name);
				let mut foreign_rpk_dir = foreign_rpk_path.clone();
				foreign_rpk_dir.set_file_name(foreign_rpk_path.file_prefix().unwrap());

				// NOTE: package file takes priority if there is a file and folder with same name
				if foreign_rpk_path.is_file() {
					let foreign_file = fs::File::open(&foreign_rpk_path)
						.await
						.expect("error while opening mod file");
					let time = foreign_file
						.metadata()
						.await
						.unwrap()
						.modified()
						.unwrap()
						.duration_since(UNIX_EPOCH)
						.unwrap()
						.as_secs();
					metadata.insert(foreign_rpk_path, time);

					let mut buf_reader = io::BufReader::new(foreign_file);
					let mut buffer = Vec::new();
					buf_reader.read_to_end(&mut buffer).await.unwrap();
					let mut cursor = std::io::Cursor::new(buffer);
					let mut reader = Reader::new(&mut cursor);

					if let Format::Rpk(foreign_package) =
						Format::from_reader_with_ctx(&mut reader, ())
							.expect("error while reading mod format")
					{
						for (i, foreign_entry) in foreign_package.entries.iter().enumerate() {
							if let Some(j) = native_entries
								.iter()
								.position(|native_entry| native_entry.name == foreign_entry.name)
							{
								let foreign_data = foreign_package.data.get(i).unwrap();
								let native_data = native_package.data.get_mut(j).unwrap();
								*native_data = foreign_data.clone();
							} else {
								// TODO: Verify this works
								// add the mod's entry to exanima's rpk file
								native_entries.push(foreign_entry.clone());
								native_package.data.push(foreign_package.data[i].clone());
							}
						}
					}
				} else if foreign_rpk_dir.is_dir() {
					let mut read_foreign_dir = fs::read_dir(foreign_rpk_dir).await.unwrap();
					while let Some(foreign_entry) = read_foreign_dir.next_entry().await.unwrap() {
						let foreign_entry_path = foreign_entry.path();
						let foreign_file = fs::File::open(&foreign_entry_path).await.unwrap();
						let time = foreign_file
							.metadata()
							.await
							.unwrap()
							.modified()
							.unwrap()
							.duration_since(UNIX_EPOCH)
							.unwrap()
							.as_secs();
						metadata.insert(foreign_entry_path, time);

						let mut reader = io::BufReader::new(foreign_file);
						let mut foreign_data = Vec::new();
						reader.read_to_end(&mut foreign_data).await.unwrap();
						if let Some(j) = native_entries.iter().position(|native_entry| {
							native_entry.name == foreign_entry.file_name().display().to_string()
						}) {
							let native_data = native_package.data.get_mut(j).unwrap();
							*native_data = foreign_data;
						} else {
							// TODO: Verify this works
							// add the mod's entry to exanima's rpk file
							native_entries.push(rpk::Entry {
								name: foreign_entry.file_name().display().to_string(),
								// default is used here as offset and size are computed later
								..Default::default()
							});
							native_package.data.push(foreign_data);
						}
					}
				}
			}
			let mut previous_offset = 0;
			let mut previous_size = 0;
			for (i, native_data) in native_package.data.iter().enumerate() {
				let native_entry = native_entries.get_mut(i).unwrap();
				native_entry.offset = previous_offset + previous_size;
				native_entry.size = native_data.len() as u32;
				previous_offset = native_entry.offset;
				previous_size = native_entry.size;
			}
			native_entries.sort_by(|a, b| a.name.cmp(&b.name));
			native_package.entries = native_entries;
		};

		let cache_file_path = cache_build_dir.join(native_rpk_name);
		if !cache_build_dir.is_dir() {
			fs::create_dir_all(&cache_build_dir)
				.await
				.expect("error while creating cache directory");
		}
		let mut cache_buf_writer = std::io::BufWriter::new(
			std::fs::File::create(cache_file_path).expect("error while creating cache file"),
		);
		let mut cache_writer = Writer::new(&mut cache_buf_writer);
		native_rpk_format
			.to_writer(&mut cache_writer, ())
			.expect("error while serializing to cache file");
	}

	let buffer = ron::ser::to_string_pretty(&metadata, ron::ser::PrettyConfig::default()).unwrap();
	let mut writer = io::BufWriter::new(fs::File::create(metadata_path).await.unwrap());
	writer.write_all(buffer.as_bytes()).await.unwrap();
	writer.flush().await.unwrap();

	let buffer = ron::to_string(&load_order).unwrap();
	let mut writer = io::BufWriter::new(fs::File::create(cache_load_order_path).await.unwrap());
	writer.write_all(buffer.as_bytes()).await.unwrap();
	writer.flush().await.unwrap();
}
