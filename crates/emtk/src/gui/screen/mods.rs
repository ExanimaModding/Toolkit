use std::{collections::HashMap, path::PathBuf};

use emf_types::config::PluginConfig;
use iced::{
	advanced::widget,
	widget::{checkbox, container, horizontal_space, scrollable, svg, text, Column, Row},
	Alignment, Border, Element, Length, Point, Rectangle, Task, Theme,
};
use iced_drop::{droppable, find_zones};

use crate::{
	config,
	gui::{config_by_id, load_order, theme, Icon},
};

#[derive(Debug, Clone)]
pub enum Action {
	SettingsChanged(config::Settings),
	None,
}

// TODO: implement notify-rs to watch for new/deleted mods and changes to config.toml
// https://github.com/notify-rs/notify
// TODO: implement mod conflict detection
// TODO: support dragging multiple mods via multi-select
#[derive(Debug, Default, Clone)]
pub struct Mods {
	hovered_mod: Option<widget::Id>,
	load_order: Vec<(widget::Id, container::Id, PluginConfig)>,
	settings: config::Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
	ModDragCanceled,
	ModDragged(usize, Point, Rectangle),
	ModDropped(usize, Point, Rectangle),
	ModToggled(usize, bool),
	ModZonesFound(Vec<(widget::Id, Rectangle)>),
	SettingsRefetched(config::Settings),
}

impl Mods {
	pub fn new(mut settings: config::Settings) -> (Self, Action) {
		match &settings.exanima_exe {
			Some(path) => {
				let path = PathBuf::from(path);
				settings.load_order = load_order(&path);
				(
					Self {
						load_order: settings
							.load_order
							.iter()
							.map(|(mod_id, _enabled)| {
								let container_id = container::Id::new(mod_id.clone());
								(
									widget::Id::from(container_id.clone()),
									container_id,
									config_by_id(&path, mod_id).unwrap(),
								)
							})
							.collect(),
						settings: settings.clone(),
						..Default::default()
					},
					Action::SettingsChanged(settings),
				)
			}
			None => (
				Self {
					settings,
					..Default::default()
				},
				Action::None,
			),
		}
	}

	pub fn update(&mut self, message: Message) -> (Task<Message>, Action) {
		match message {
			Message::ModDragCanceled => self.hovered_mod = None,
			Message::ModDragged(_index, _point, rectangle) => {
				return (
					find_zones(
						Message::ModZonesFound,
						move |bounds| bounds.intersects(&rectangle),
						Some(
							self.load_order
								.iter()
								.map(|(widget_id, _container_id, _mod_id)| widget_id.clone())
								.collect(),
						),
						None,
					),
					Action::None,
				);
			}
			Message::ModDropped(from_index, _point, _rectangle) => {
				if let Some(to_id) = &self.hovered_mod
					&& let Some(to_index) = self.load_order.iter().enumerate().find_map(
						move |(index, (widget_id, _, _))| {
							if to_id == widget_id {
								Some(index)
							} else {
								None
							}
						},
					) {
					let (widget_id, container_id, mod_id) = self.load_order[from_index].clone();
					self.load_order.remove(from_index);
					self.load_order
						.insert(to_index, (widget_id, container_id, mod_id.clone()));
					let (mod_id, enabled) = self.settings.load_order[from_index].clone();
					self.settings.load_order.remove(from_index);
					self.settings.load_order.insert(to_index, (mod_id, enabled));

					self.hovered_mod = None;
					return (Task::none(), Action::SettingsChanged(self.settings.clone()));
				}
			}
			Message::ModToggled(index, enabled) => {
				self.settings.load_order[index].1 = enabled;
				return (Task::none(), Action::SettingsChanged(self.settings.clone()));
			}
			Message::ModZonesFound(zones) => {
				if let Some(zone) = zones.first() {
					self.hovered_mod = Some(zone.0.clone())
				}
			}
			Message::SettingsRefetched(settings) => {
				self.load_order = settings
					.load_order
					.iter()
					.map(|(mod_id, _enabled)| {
						let container_id = container::Id::new(mod_id.clone());
						(
							widget::Id::from(container_id.clone()),
							container_id,
							config_by_id(
								&PathBuf::from(self.settings.exanima_exe.as_ref().unwrap()),
								mod_id,
							)
							.unwrap(),
						)
					})
					.collect();
				self.settings = settings;
			}
		}

		(Task::none(), Action::None)
	}

	pub fn view(&self, icons: &HashMap<Icon, svg::Handle>) -> Element<Message> {
		let name_column = Length::FillPortion(12);
		let version_column = Length::FillPortion(1);
		container(
			container(
				Column::new()
					.push(
						container(
							Row::new()
								.push(horizontal_space().width(Length::Fixed(23.)))
								.push(text("Name").center().width(name_column))
								.push(text("Version").center().width(version_column)),
						)
						.padding(6)
						.width(Length::Fill),
					)
					.push(
						container(scrollable(Column::with_children(
							self.load_order.iter().enumerate().map(
								|(index, (widget_id, container_id, config))| {
									droppable(
										container(
											Row::new()
												.push(
													svg(icons.get(&Icon::Menu).unwrap().clone())
														.width(Length::Shrink)
														.style(theme::svg),
												)
												.push(
													container(
														checkbox(
															// "",
															config.plugin.name.clone(),
															self.settings.load_order[index].1,
														)
														.on_toggle(move |enabled| {
															Message::ModToggled(index, enabled)
														}),
													)
													.width(name_column),
												)
												.push(
													container(text(config.plugin.version.clone()))
														.align_x(Alignment::Center)
														.width(version_column),
												)
												.align_y(Alignment::Center)
												.spacing(3),
										)
										.id(container_id.clone())
										.padding(4)
										.style(move |theme: &Theme| {
											let palette = theme.extended_palette();
											let style = container::Style::default();
											if let Some(to_id) = &self.hovered_mod
												&& widget_id == to_id
											{
												style.background(palette.primary.weak.color)
											} else if index % 2 == 0 {
												style.background(palette.background.weak.color)
											} else {
												style
											}
										}),
									)
									.on_drag(move |point, rectangle| {
										Message::ModDragged(index, point, rectangle)
									})
									.on_drop(move |point, rectangle| {
										Message::ModDropped(index, point, rectangle)
									})
									.on_cancel(Message::ModDragCanceled)
									.drag_hide(true)
									.into()
								},
							),
						)))
						.width(Length::Fill)
						.height(Length::Fill)
						.padding(2),
					),
			)
			.style(|theme| {
				container::Style::default().border(
					Border::default()
						.color(theme.extended_palette().background.strong.color)
						.width(2),
				)
			}),
		)
		.padding(12)
		.into()
	}
}
