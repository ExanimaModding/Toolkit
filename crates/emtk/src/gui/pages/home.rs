use crate::config::AppSettings;

use iced::{
	widget::{container, horizontal_rule, text, Column},
	Element, Length, Task,
};

#[derive(Debug, Clone)]
pub enum Message {
	OpenUrl(String),
	LoadSettings(crate::config::AppSettings),
	UpdateModOrder,
	UpdateModSettings(bool),
}

#[derive(Debug, Default, Clone)]
pub struct Home {
	settings: AppSettings,
}

impl Home {
	pub fn update(
		&mut self,
		_app_state: &mut crate::gui::state::AppState,
		message: Message,
	) -> Task<crate::gui::Message> {
		match message {
			Message::OpenUrl(url) => {
				log::info!("Opening URL: {}", url);
				open::that(url).unwrap();
				Task::none()
			}
			Message::LoadSettings(settings) => {
				self.settings = settings;

				Task::none()
			}
			Message::UpdateModOrder => Task::none(),
			Message::UpdateModSettings(mod_toggle) => {
				// TODO: save settings
				Task::none()
			}
		}
		.map(crate::gui::Message::HomePage)
	}

	pub fn view(&self) -> Element<Message> {
		container(
			Column::new()
				.push(text("Welcome to the Exanima Modding Toolkit Launcher!").size(20))
				.push(horizontal_rule(1))
				.push(self.mods_list())
				.spacing(10),
		)
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
