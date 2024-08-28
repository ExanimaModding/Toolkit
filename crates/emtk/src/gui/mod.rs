mod constants;
mod menu;
mod modal;
mod pages;
mod sidebar;
mod state;

use iced::{
	widget::{container, horizontal_rule, text, Column, Row},
	Element, Length, Padding, Task, Theme,
};
use modal::ModalKind;

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

#[derive(Debug, Clone)]
pub enum Message {
	HomePage(pages::home::Message),
	Menu(menu::Message),
	Modal,
	ModalLaunching,
	ModalTest,
	Settings(pages::settings::Message),
	Sidebar(sidebar::Message),
}

#[derive(Debug, Default, Clone)]
pub struct Emtk {
	sidebar: sidebar::Sidebar,
	menu: menu::Menu,

	home_page: pages::home::Home,
	settings: pages::settings::Settings,

	app_state: state::AppState,
	modal: Option<ModalKind>,
}

impl Emtk {
	pub fn new() -> (Self, Task<Message>) {
		let state = Self::default();
		let settings = state.app_state.settings.clone();
		(
			state,
			Task::batch([
				Task::done(pages::settings::Message::default()).map(Message::Settings),
				Task::done(pages::home::Message::LoadSettings(settings.clone()))
					.map(Message::HomePage),
				Task::done(sidebar::Message::LoadSettings(settings)).map(Message::Sidebar),
			]),
		)
	}

	pub fn theme(_state: &Emtk) -> Theme {
		Theme::CatppuccinFrappe
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::HomePage(message) => self.home_page.update(&mut self.app_state, message),
			Message::Menu(message) => self.menu.update(&mut self.app_state, message),
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
			Message::Sidebar(message) => self.sidebar.update(message),
			Message::Settings(message) => self.settings.update(&mut self.app_state, message),
		}
	}

	pub fn view(&self) -> Element<Message> {
		let con = container(
			Row::new()
				.spacing(10.)
				.push(
					Column::new()
						.push(self.sidebar.view().map(Message::Sidebar))
						.width(Length::Fixed(256.)),
				)
				.push(Column::new().push(self.page()).width(Length::Fill)),
		)
		.padding(Padding::new(12.0));

		if let Some(modal) = &self.modal {
			modal::modal(con, modal.view(), Message::Modal)
		} else {
			con.into()
		}
	}

	pub fn page(&self) -> Element<Message> {
		let page: Element<Message> = match self.menu.current_page {
			menu::Page::Home => self.home_page.view().map(Message::HomePage),
			menu::Page::Settings => self.settings.view().map(Message::Settings),
		};

		container(
			Column::new()
				.push(text("Exanima Modding Toolkit").size(30))
				.push(horizontal_rule(1.))
				.push(
					Column::new()
						.push(self.menu.view().map(Message::Menu))
						.push(page),
				),
		)
		.into()
	}
}
