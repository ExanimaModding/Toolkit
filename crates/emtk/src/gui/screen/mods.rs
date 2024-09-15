use crate::config::AppSettings;

use iced::{
	widget::{container, horizontal_rule, text, Column},
	Element, Length,
};

#[derive(Debug, Default, Clone)]
pub struct Mods {
	settings: AppSettings,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl Mods {
	pub fn update(&mut self, message: Message) {}

	pub fn view(&self) -> Element<Message> {
		container(
			Column::new()
				.push(text("Mods").size(36))
				.push(horizontal_rule(1))
				.push(self.mods_list())
				.spacing(6),
		)
		.padding(12)
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}

	fn mods_list(&self) -> Element<Message> {
		container(text("WIP"))
			.width(Length::Fill)
			.height(Length::Fill)
			.into()
	}
}
