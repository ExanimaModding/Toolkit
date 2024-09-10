use crate::config::AppSettings;

use iced::{
	widget::{container, horizontal_rule, text, Column},
	Element, Length, Task,
};

#[derive(Debug, Default, Clone)]
pub struct Home {
	settings: AppSettings,
}

#[derive(Debug, Clone)]
pub enum Message {
	LoadSettings(crate::config::AppSettings),
	ModOrderUpdated,
	ModSettingsUpdated(bool),
	UrlOpened(String),
}

impl Home {
	pub fn update(
		&mut self,
		message: Message,
		_app_state: &mut crate::gui::state::AppState,
	) -> Task<Message> {
		match message {
			Message::LoadSettings(settings) => self.settings = settings,
			Message::ModOrderUpdated => (),
			// TODO: save settings
			Message::ModSettingsUpdated(_mod_toggle) => (),
			Message::UrlOpened(url) => {
				log::info!("Opening URL: {}", url);
				open::that(url).unwrap();
			}
		};

		Task::none()
	}

	pub fn view(&self) -> Element<Message> {
		container(
			Column::new()
				.push(text("Exanima Modding Toolkit Launcher").size(36))
				.push(horizontal_rule(1))
				.push(self.mods_list())
				.spacing(10),
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
