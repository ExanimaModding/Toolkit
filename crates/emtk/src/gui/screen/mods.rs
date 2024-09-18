use std::path::PathBuf;

use crate::{
	config,
	gui::{config_by_id, load_order},
};

use emf_types::config::PluginConfig;
use iced::{
	advanced::widget::Id,
	widget::{checkbox, container, horizontal_rule, scrollable, text, Column, Row},
	Border, Element, Length, Point, Rectangle, Task, Theme,
};
use iced_drop::{droppable, find_zones};

#[derive(Debug, Clone)]
pub enum Action {
	SettingsChanged(config::Settings),
	None,
}

// TODO: implement notify-rs to watch for new/deleted mods and changes to config.toml
// https://github.com/notify-rs/notify
#[derive(Debug, Default, Clone)]
pub struct Mods {
	hovered_mod: Option<Id>,
	load_order: Vec<(Id, container::Id, PluginConfig)>,
	settings: config::Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
	ModDragged(usize, Point, Rectangle),
	ModDropped(usize, Point, Rectangle),
	ModToggled(usize, bool),
	ModZonesFound(Vec<(Id, Rectangle)>),
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
									Id::from(container_id.clone()),
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
							Id::from(container_id.clone()),
							container_id,
							config_by_id(
								&PathBuf::from(self.settings.exanima_exe.as_ref().unwrap()),
								&mod_id,
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

	pub fn view(&self) -> Element<Message> {
		container(
			Column::new()
				.push(
					Column::new()
						.push(text("Mods").size(36))
						.push(horizontal_rule(1))
						.spacing(6),
				)
				.push(scrollable(Column::with_children(
					self.load_order.iter().enumerate().map(
						|(index, (_widget_id, container_id, config))| {
							container(
								Row::new()
									.push(
										checkbox("", self.settings.load_order[index].1).on_toggle(
											move |enabled| Message::ModToggled(index, enabled),
										),
									)
									.push(
										droppable(
											container(
												text(config.plugin.name.clone())
													.width(Length::Fill),
											)
											.id(container_id.clone()),
										)
										.on_drag(move |point, rectangle| {
											Message::ModDragged(index, point, rectangle)
										})
										.on_drop(move |point, rectangle| {
											Message::ModDropped(index, point, rectangle)
										})
										.drag_hide(true),
									)
									.padding(4),
							)
							.style(|theme: &Theme| {
								container::Style::default().border(
									Border::default()
										.color(theme.extended_palette().background.strong.color)
										.width(2.)
										// .rounded(3.),
								)
							})
							.into()
						},
					),
				)))
				.spacing(12),
		)
		.padding(12)
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}
}
