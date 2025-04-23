use iced::{Element, Fill};

use crate::gui::{log, widget::scrollable, Root};

#[derive(Debug, Default, Clone)]
pub struct Logs;

#[derive(Debug, Clone)]
pub enum Message {}

impl Logs {
	pub fn update(&mut self, _message: Message) {}

	pub fn view(&self, root: &Root) -> Element<Message> {
		let content = log::view(&root.logs);

		scrollable(content)
			.anchor_bottom()
			.width(Fill)
			.height(Fill)
			.into()
	}

	pub fn title(&self) -> String {
		"Logs".to_string()
	}
}
