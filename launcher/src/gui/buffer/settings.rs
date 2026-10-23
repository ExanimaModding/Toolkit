use std::{env, io};

use emtk::Error;
use iced::{
    Alignment, Element, Fill, Task, Theme,
    widget::{column, container, pick_list, row, text},
};
use log::error;

use super::Root;
use crate::gui::{
    default_theme,
    widget::{button, default_value, icon},
};

pub enum Action {
    None,
    ThemeSelected(Theme),
    Task(Task<Message>),
}

#[derive(Debug)]
pub struct Settings;

#[derive(Debug, Clone)]
pub enum Message {
    OpenAppDataDir,
    ThemeSelected(Theme),
}

impl Settings {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::OpenAppDataDir => {
                let Some(data_dir) = emtk::data_dir() else {
                    error!("{}", io::Error::from(io::ErrorKind::NotFound));
                    return Action::None;
                };
                Action::Task(
                    Task::future(async move {
                        let _ = open::that_in_background(data_dir)
                            .join()
                            .map(|r| {
                                r.map_err(Error::msg("failed to open app data directory"))
                                    .map_err(|e| error!("{}", e))
                            })
                            .map_err(|_| error!("failed to open app data directory in background"));
                    })
                    .discard(),
                )
            }
            Message::ThemeSelected(theme) => Action::ThemeSelected(theme),
        }
    }

    pub fn view(&self, root: &Root) -> Element<'_, Message> {
        let theme_picker = column![text("Theme").size(20), self.theme_picker(root)];

        let app_data_btn = button(
            row![
                icon::square_arrow_out_up_right().center(),
                text("Open app data directory").center()
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

    fn theme_picker(&self, root: &Root) -> Element<'_, Message> {
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

    pub fn title(&self) -> String {
        "Settings".to_string()
    }
}
