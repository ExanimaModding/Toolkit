mod constants;
mod modal;
mod screen;
mod sidebar;
mod state;

use iced::{
	widget::{container, Column, Row},
	Element, Length, Padding, Task, Theme,
};
use modal::ModalKind;
use screen::{
	home::{self, Home},
	settings::{self, Settings},
	Screen, ScreenKind,
};
use sidebar::Sidebar;

static ICON: &[u8] = include_bytes!("../../../../assets/images/corro.ico");

pub(crate) async fn start_gui() -> iced::Result {
	let image = image::load_from_memory(ICON).unwrap();
	let icon =
		iced::window::icon::from_rgba(image.as_bytes().to_vec(), image.height(), image.width())
			.unwrap();

	iced::application("Exanima Modding Toolkit", Emtk::update, Emtk::view)
		.theme(Emtk::theme)
		.window(iced::window::Settings {
			icon: Some(icon),
			..Default::default()
		})
		.run_with(Emtk::new)
}

#[derive(Debug, Default, Clone)]
pub struct Emtk {
	sidebar: Sidebar,
	app_state: state::AppState,
	modal: Option<ModalKind>,
	screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Message {
	Home(home::Message),
	Modal,
	ModalLaunching,
	ModalTest,
	ScreenChanged(ScreenKind),
	Settings(settings::Message),
	Sidebar(sidebar::Message),
}

impl Emtk {
	pub fn new() -> (Self, Task<Message>) {
		let emtk = Self::default();
		let settings = emtk.app_state.settings.clone();
		(
			emtk,
			// TODO: refactor
			Task::batch([
				Task::done(screen::settings::Message::default()).map(Message::Settings),
				Task::done(screen::home::Message::LoadSettings(settings.clone()))
					.map(Message::Home),
				Task::done(sidebar::Message::LoadSettings(settings)).map(Message::Sidebar),
			]),
		)
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::Home(message) => match &mut self.screen {
				Screen::Home(screen) => screen.update(&mut self.app_state, message),
				_ => Task::none(),
			},
			Message::Modal => {
				self.modal = None;
				Task::none()
			}
			Message::ModalLaunching => {
				self.modal = Some(ModalKind::Launching);
				Task::none()
			}
			Message::ModalTest => {
				self.modal = Some(ModalKind::Test);
				Task::none()
			}
			Message::ScreenChanged(kind) => match kind {
				ScreenKind::Home => {
					self.screen = Screen::Home(Home::default());
					Task::none()
				}
				ScreenKind::Settings => {
					self.screen = Screen::Settings(Settings::default());
					Task::none()
				}
			},
			Message::Sidebar(message) => self.sidebar.update(message),
			Message::Settings(message) => match &mut self.screen {
				Screen::Settings(screen) => screen.update(&mut self.app_state, message),
				_ => Task::none(),
			},
		}
	}

	pub fn view(&self) -> Element<Message> {
		let screen = match &self.screen {
			Screen::Home(screen) => screen.view().map(Message::Home),
			Screen::Settings(screen) => screen.view().map(Message::Settings),
		};

		let con = container(
			Row::new()
				.spacing(10.)
				.push(
					Column::new()
						.push(self.sidebar.view().map(Message::Sidebar))
						.width(Length::Fixed(256.)),
				)
				.push(Column::new().push(screen).width(Length::Fill)),
		)
		.padding(Padding::new(12.0));

		if let Some(modal) = &self.modal {
			modal::modal(con, modal.view(), Message::Modal)
		} else {
			con.into()
		}
	}

	pub fn theme(_state: &Emtk) -> Theme {
		Theme::CatppuccinFrappe
	}
}
