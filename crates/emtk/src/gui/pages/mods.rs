use iced::{
	widget::{Column, Row, Text},
	Element, Task,
};

use crate::config::AppSettings;
#[derive(Debug, Clone)]
pub enum Message {
	LoadSettings(crate::config::AppSettings),
	UpdateModOrder,
	UpdateModSettings,
}

#[derive(Debug, Default, Clone)]
pub struct Mods {
	unsaved_settings: AppSettings,
}

impl Mods {
	pub fn view(&self) -> Element<Message> {
		self.mods_list()
	}

	fn mods_list(&self) -> Element<Message> {
		let mut list: Column<_> = Column::new();

		for mod_setting in self.unsaved_settings.mods.iter() {
			let mod_name = Text::new(&mod_setting.info.config.plugin.name);
			let mod_enabled = Text::new(if mod_setting.info.config.plugin.enabled {
				"Enabled"
			} else {
				"Disabled"
			});

			list = list.push(Row::new().push(mod_name).push(mod_enabled));
		}

		list.into()
	}

	pub fn update(
		&mut self,
		_app_state: &mut crate::gui::state::AppState,
		message: Message,
	) -> Task<crate::gui::Message> {
		match message {
			Message::LoadSettings(settings) => {
				self.unsaved_settings = settings;
				Task::none()
			}
			_ => Task::none(),
		}
	}
}
