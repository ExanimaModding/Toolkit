pub mod instance;
pub mod instance_history;
pub mod logs;
pub mod settings;

use std::path::PathBuf;

use iced::{
	widget::{column, container, row, text},
	Element, Fill, Padding, Task,
};

use crate::gui::{
	widget::{button, icon, tooltip},
	Root,
};
use instance::Instance;
use instance_history::InstanceHistory;
use logs::Logs;
use settings::Settings;

pub enum Action {
	Instance(instance::Action),
	InstanceHistory(instance_history::Action),
	Loaded,
	Loading,
	NewInstance,
	NewInstanceHistory,
	NewLogs,
	NewSettings,
	None,
	OpenInstance(PathBuf),
	Settings(settings::Action),
	Task(Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Buffer {
	Instance(Box<Instance>),
	InstanceHistory(Box<InstanceHistory>),
	Logs(Box<Logs>),
	Settings(Box<Settings>),
}

impl From<Instance> for Buffer {
	fn from(value: Instance) -> Self {
		Buffer::Instance(Box::new(value))
	}
}

impl From<InstanceHistory> for Buffer {
	fn from(value: InstanceHistory) -> Self {
		Buffer::InstanceHistory(Box::new(value))
	}
}

impl From<Logs> for Buffer {
	fn from(value: Logs) -> Self {
		Buffer::Logs(Box::new(value))
	}
}

impl From<Settings> for Buffer {
	fn from(value: Settings) -> Self {
		Buffer::Settings(Box::new(value))
	}
}

#[derive(Debug, Clone)]
pub enum Message {
	ImportInstanceDialog,
	Instance(instance::Message),
	InstanceHistory(instance_history::Message),
	Loaded,
	Loading,
	Logs(logs::Message),
	NewInstance,
	NewInstanceHistory,
	NewLogs,
	NewSettings,
	OpenInstance(PathBuf),
	Settings(settings::Message),
}

impl Buffer {
	pub fn update(&mut self, message: Message) -> Action {
		match (self, message) {
			(_, Message::ImportInstanceDialog) => {
				return Action::Task(
					Task::done(Message::Loading).chain(
						Task::future(rfd::AsyncFileDialog::new().pick_folder())
							.and_then(|handle| {
								Task::done(Message::OpenInstance(handle.path().into()))
							})
							.chain(Task::done(Message::Loaded)),
					),
				)
			}
			(Buffer::Instance(instance), Message::Instance(message)) => {
				return match instance.update(message) {
					instance::Action::Task(task) => Action::Task(task.map(Message::Instance)),
					action => Action::Instance(action),
				}
			}
			(Buffer::InstanceHistory(instance_history), Message::InstanceHistory(message)) => {
				return match instance_history.update(message) {
					instance_history::Action::Task(task) => {
						Action::Task(task.map(Message::InstanceHistory))
					}
					action => Action::InstanceHistory(action),
				};
			}
			(_, Message::Loaded) => return Action::Loaded,
			(_, Message::Loading) => return Action::Loading,
			(Buffer::Logs(logs), Message::Logs(message)) => logs.update(message),
			(_, Message::NewInstance) => return Action::NewInstance,
			(_, Message::NewInstanceHistory) => return Action::NewInstanceHistory,
			(_, Message::NewLogs) => return Action::NewLogs,
			(_, Message::NewSettings) => return Action::NewSettings,
			(_, Message::OpenInstance(path)) => return Action::OpenInstance(path),
			(Buffer::Settings(settings), Message::Settings(message)) => {
				return Action::Settings(settings.update(message));
			}
			_ => (),
		}

		Action::None
	}

	pub fn view(&self, root: &Root) -> Element<Message> {
		let content = container(match self {
			Buffer::Instance(instance) => instance.view().map(Message::Instance),
			Buffer::InstanceHistory(instance_history) => {
				instance_history.view().map(Message::InstanceHistory)
			}
			Buffer::Logs(logs) => logs.view(root).map(Message::Logs),
			Buffer::Settings(settings) => settings.view(root).map(Message::Settings),
		})
		.padding(Padding::default().top(8))
		.style(|theme| container::Style {
			background: Some(theme.palette().background.into()),
			..Default::default()
		});

		column![self.controls(), content].spacing(8).into()
	}

	fn controls(&self) -> Element<Message> {
		let icon_size = 20;
		let btn_size = 38;
		container(
			row![
				tooltip(
					button(icon::folder_open().size(icon_size).center())
						.on_press(Message::ImportInstanceDialog)
						.width(btn_size)
						.height(btn_size),
					text("Import instance"),
					tooltip::Position::Bottom
				),
				tooltip(
					button(icon::folder().size(icon_size).center())
						.on_press(Message::NewInstance)
						.width(btn_size)
						.height(btn_size),
					text("Open recent instance"),
					tooltip::Position::Bottom
				),
				tooltip(
					button(icon::folders().size(icon_size).center())
						.on_press(Message::NewInstanceHistory)
						.width(btn_size)
						.height(btn_size),
					text("Instance history"),
					tooltip::Position::Bottom
				),
				tooltip(
					button(icon::logs().size(icon_size).center())
						.on_press(Message::NewLogs)
						.width(btn_size)
						.height(btn_size),
					text("Logs"),
					tooltip::Position::Bottom
				),
				tooltip(
					button(icon::settings().size(icon_size).center())
						.on_press(Message::NewSettings)
						.width(btn_size)
						.height(btn_size),
					text("Settings"),
					tooltip::Position::Bottom
				)
			]
			.spacing(3),
		)
		.padding([0, 8])
		.into()
	}

	pub fn title(&self) -> String {
		match self {
			Buffer::Instance(instance) => instance.title(),
			Buffer::InstanceHistory(instance_history) => instance_history.title(),
			Buffer::Logs(logs) => logs.title(),
			Buffer::Settings(settings) => settings.title(),
		}
	}
}
