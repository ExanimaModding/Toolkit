mod constants;
mod menu;
mod pages;
mod sidebar;
mod state;

use iced::{
	widget::{container, horizontal_rule, text, Column, Row},
	Element, Length, Padding, Task, Theme,
};

static ICON: &[u8] = include_bytes!("../../../../assets/images/corro.ico");

#[derive(Debug, Clone)]
pub enum Message {
	Menu(menu::Message),
	HomePage(pages::home::Message),
	Settings(pages::settings::Message),
	Sidebar(sidebar::Message),
}

#[derive(Debug, Default, Clone)]
pub struct State {
	sidebar: sidebar::Sidebar,
	menu: menu::Menu,

	home_page: pages::home::Home,
	settings: pages::settings::Settings,

	app_state: state::AppState,
}

impl State {
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

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::HomePage(message) => self.home_page.update(&mut self.app_state, message),
			Message::Settings(message) => self.settings.update(&mut self.app_state, message),
			Message::Menu(message) => self.menu.update(&mut self.app_state, message),
			Message::Sidebar(message) => self.sidebar.update(message),
		}
	}

	pub fn view(&self) -> Element<Message> {
		container(
			Row::new()
				.spacing(10.)
				.push(
					Column::new()
						.push(
							self.sidebar
								.view()
								// TODO: remove explain
								.explain(iced::Color::BLACK)
								.map(Message::Sidebar),
						)
						.width(Length::Fixed(256.)),
				)
				.push(Column::new().push(self.page()).width(Length::Fill)),
		)
		.padding(Padding::new(12.0))
		.into()
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

fn theme(_state: &State) -> Theme {
	Theme::CatppuccinFrappe
}

pub(crate) async fn start_gui() -> iced::Result {
	let image = image::load_from_memory(ICON).unwrap();
	let icon =
		iced::window::icon::from_rgba(image.as_bytes().to_vec(), image.height(), image.width())
			.unwrap();

	iced::application("Exanima Modding Toolkit", State::update, State::view)
		.theme(theme)
		.window(iced::window::Settings {
			icon: Some(icon),
			..Default::default()
		})
		.run_with(State::new)
}
