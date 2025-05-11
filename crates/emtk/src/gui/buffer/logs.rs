use iced::{
	Element, Fill, Padding,
	widget::{container, scrollable},
};
use tracing::instrument;

use crate::gui::{Root, log};

#[derive(Debug, Default, Clone)]
pub struct Logs;

#[derive(Debug, Clone)]
pub enum Message {}

impl Logs {
	#[instrument(level = "trace")]
	pub fn update(&mut self, _message: Message) {}

	#[instrument(level = "trace")]
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

	#[instrument(level = "trace")]
	pub fn title(&self) -> String {
		"Logs".to_string()
	}
}
