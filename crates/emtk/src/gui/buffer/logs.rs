use iced::{
	Element, Fill, Padding,
	widget::{container, scrollable},
};

use crate::gui::{Root, log};

#[derive(Debug, Default, Clone)]
pub struct Logs;

#[derive(Debug, Clone)]
pub enum Message {}

impl Logs {
	pub fn update(&mut self, _message: Message) {}

	pub fn view(&self, root: &Root) -> Element<Message> {
		let content = log::view(&root.logs);

		scrollable(container(content).padding(Padding::default().bottom(10).right(10)))
			.direction(scrollable::Direction::Both {
				vertical: scrollable::Scrollbar::default(),
				horizontal: scrollable::Scrollbar::default(),
			})
			.anchor_bottom()
			.width(Fill)
			.height(Fill)
			.into()
	}

	pub fn title(&self) -> String {
		"Logs".to_string()
	}
}
