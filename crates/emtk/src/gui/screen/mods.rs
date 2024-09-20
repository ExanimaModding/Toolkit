use std::{collections::HashMap, path::PathBuf};

use emf_types::config::PluginConfig;
use iced::{
	advanced::widget,
	widget::{checkbox, container, horizontal_space, scrollable, svg, text, Column, Row},
	Alignment, Border, Element, Length, Point, Rectangle, Task, Theme,
};
use iced_drop::{droppable, find_zones};

use crate::{
	config::Config,
	gui::{config_by_id, theme, Icon},
};

#[derive(Debug, Clone)]
pub enum Action {
	ConfigChanged(Config),
	None,
}

#[derive(Debug, Clone)]
pub struct ModView {
	widget_id: widget::Id,
	container_id: container::Id,
	mod_config: Option<PluginConfig>,
}

impl ModView {
	pub fn new(
		widget_id: widget::Id,
		container_id: container::Id,
		mod_config: Option<PluginConfig>,
	) -> Self {
		Self {
			widget_id,
			container_id,
			mod_config,
		}
	}
}

// TODO: implement notify-rs to watch for new/deleted mods and changes to config.toml
// https://github.com/notify-rs/notify
// TODO: implement mod conflict detection
// TODO: support dragging multiple mods via multi-select
#[derive(Debug, Default, Clone)]
pub struct Mods {
	config: Config,
	hovered_mod: Option<widget::Id>,
	load_order: Vec<ModView>,
}

#[derive(Debug, Clone)]
pub enum Message {
	ConfigRefetched(Config),
	ModDragCanceled,
	ModDragged(usize, Point, Rectangle),
	ModDropped(usize, Point, Rectangle),
	ModToggled(usize, bool),
	ModZonesFound(Vec<(widget::Id, Rectangle)>),
}

impl Mods {
	pub fn new(config: Config) -> Self {
		match &config.exanima_exe {
			Some(path) => {
				let path = PathBuf::from(path);

				Self {
					config: config.clone(),
					load_order: config
						.load_order
						.iter()
						.map(|(mod_id, _enabled)| {
							let container_id = container::Id::new(mod_id.clone());
							ModView::new(
								widget::Id::from(container_id.clone()),
								container_id,
								config_by_id(&path, mod_id),
							)
						})
						.collect(),
					..Default::default()
				}
			}
			None => Self {
				config,
				..Default::default()
			},
		}
	}

	pub fn update(&mut self, message: Message) -> (Task<Message>, Action) {
		match message {
			Message::ConfigRefetched(config) => {
				if let Some(exanima_exe) = &config.exanima_exe {
					self.load_order = config
						.load_order
						.iter()
						.map(|(mod_id, _enabled)| {
							let path = PathBuf::from(exanima_exe);
							let container_id = container::Id::new(mod_id.clone());

							ModView::new(
								widget::Id::from(container_id.clone()),
								container_id,
								config_by_id(&path, mod_id),
							)
						})
						.collect();
				}
				self.config = config;
			}
			Message::ModDragCanceled => self.hovered_mod = None,
			Message::ModDragged(_index, _point, rectangle) => {
				return (
					find_zones(
						Message::ModZonesFound,
						move |bounds| bounds.intersects(&rectangle),
						Some(
							self.load_order
								.iter()
								.map(|(mod_view)| mod_view.widget_id.clone())
								.collect(),
						),
						None,
					),
					Action::None,
				);
			}
			Message::ModDropped(from_index, _point, _rectangle) => {
				if let Some(to_id) = &self.hovered_mod
					&& let Some(to_index) =
						self.load_order
							.iter()
							.enumerate()
							.find_map(move |(index, mod_view)| {
								if to_id == &mod_view.widget_id {
									Some(index)
								} else {
									None
								}
							}) {
					let mod_view = self.load_order[from_index].clone();
					self.load_order.remove(from_index);
					self.load_order.insert(to_index, mod_view);
					let (mod_id, enabled) = self.config.load_order[from_index].clone();
					self.config.load_order.remove(from_index);
					self.config.load_order.insert(to_index, (mod_id, enabled));

					self.hovered_mod = None;
					return (Task::none(), Action::ConfigChanged(self.config.clone()));
				}
			}
			Message::ModToggled(index, enabled) => {
				self.config.load_order[index].1 = enabled;
				return (Task::none(), Action::ConfigChanged(self.config.clone()));
			}
			Message::ModZonesFound(zones) => {
				if let Some(zone) = zones.first() {
					self.hovered_mod = Some(zone.0.clone())
				}
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
							self.load_order.iter().enumerate().map(|(index, mod_view)| {
								// TODO: add tooltip conditionally for missing mods
								droppable(
									container(
										Row::new()
											.push(if mod_view.mod_config.is_some() {
												svg(icons.get(&Icon::Menu).unwrap().clone())
													.width(Length::Shrink)
													.style(theme::svg)
											} else {
												svg(icons.get(&Icon::CircleAlert).unwrap().clone())
													.width(Length::Shrink)
													.opacity(0.5)
													.style(theme::svg_danger)
											})
											.push(
												container(
													checkbox(
														if let Some(config) = &mod_view.mod_config {
															config.plugin.name.clone()
														} else {
															self.config.load_order[index].0.clone()
														},
														self.config.load_order[index].1,
													)
													.on_toggle(move |enabled| {
														Message::ModToggled(index, enabled)
													})
													.style(move |theme, status| {
														if mod_view.mod_config.is_some() {
															checkbox::primary(theme, status)
														} else {
															let mut style =
																checkbox::primary(theme, status);
															style.background =
																style.background.scale_alpha(0.5);
															style.icon_color =
																style.icon_color.scale_alpha(0.5);
															style.border = style.border.color(
																style.border.color.scale_alpha(0.5),
															);
															style.text_color = Some(
																theme
																	.palette()
																	.text
																	.scale_alpha(0.5),
															);
															style
														}
													}),
												)
												.width(name_column),
											)
											.push(
												container(
													text(
														if let Some(config) = &mod_view.mod_config {
															config.plugin.version.clone()
														} else {
															"?".to_string()
														},
													)
													.style(|theme: &Theme| {
														if mod_view.mod_config.is_some() {
															text::Style::default()
														} else {
															text::Style {
																color: Some(
																	theme
																		.palette()
																		.text
																		.scale_alpha(0.5),
																),
															}
														}
													}),
												)
												.align_x(Alignment::Center)
												.width(version_column),
											)
											.align_y(Alignment::Center)
											.spacing(3),
									)
									.id(mod_view.container_id.clone())
									.padding(4)
									.style(move |theme: &Theme| {
										let palette = theme.extended_palette();
										let style = container::Style::default();
										if let Some(to_id) = &self.hovered_mod
											&& &mod_view.widget_id == to_id
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
							}),
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
