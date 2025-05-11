use std::{env, io};

use iced::{
	Alignment, Element, Fill, Theme,
	widget::{column, container, pick_list, row, text},
};
use tracing::{error, instrument};

use super::Root;
use crate::gui::{
	default_theme,
	widget::{button, default_value, icon},
};

#[derive(Debug, Clone)]
pub enum Action {
	None,
	ThemeSelected(Theme),
}

#[derive(Debug, Clone)]
pub struct Settings;

#[derive(Debug, Clone)]
pub enum Message {
	OpenAppDataDir,
	ThemeSelected(Theme),
}

impl Settings {
	#[instrument(level = "trace")]
	pub fn update(&mut self, message: Message) -> Action {
		match message {
			Message::OpenAppDataDir => {
				let Some(data_dir) = emcore::data_dir() else {
					error!("{}", io::Error::from(io::ErrorKind::NotFound));
					return Action::None;
				};
				let _ = open::that(data_dir).map_err(|e| error!("{}", e));
			}
			Message::ThemeSelected(theme) => return Action::ThemeSelected(theme),
		}

		Action::None
	}

	#[instrument(level = "trace")]
	pub fn view(&self, root: &Root) -> Element<Message> {
		let theme_picker = column![text("Theme").size(20), self.theme_picker(root)];

		let app_data_btn = button(
			row![
				text("Open app data").center(),
				icon::square_arrow_out_up_right().size(12).center()
			]
			.align_y(Alignment::Center)
			.spacing(6),
		)
		.on_press(Message::OpenAppDataDir);

		let app_version = column![
			text("App version").size(20),
			text(format!("v{}", env!["CARGO_PKG_VERSION"]))
		];

		container(column![theme_picker, app_data_btn, app_version].spacing(12))
			.padding(6)
			.width(Fill)
			.height(Fill)
			.into()
	}

	#[instrument(level = "trace")]
	fn theme_picker(&self, root: &Root) -> Element<Message> {
		let default_theme = default_theme();
		let theme_picker = pick_list(
			Theme::ALL,
			Some(root.theme().clone()),
			Message::ThemeSelected,
		);
		if default_theme == root.theme {
			theme_picker.into()
		} else {
			default_value(theme_picker.into(), Message::ThemeSelected(default_theme)).into()
		}
	}

	#[instrument(level = "trace")]
	pub fn title(&self) -> String {
		"Settings".to_string()
	}
}
