mod constants;
mod menu;
mod pages;
mod right_panel;

use anyhow::Result;
use iced::{
	event,
	widget::{container, Button, Column, Container, Row, Rule, Text, TextInput},
	window, Element, Event, Padding, Settings, Subscription, Task, Theme,
};

static ICON: &[u8] = include_bytes!("../../../../assets/images/corro.ico");

#[derive(Debug, Clone)]
pub enum Message {
	FirstRun,
	EventOccurred(Event),
	Menu(menu::Message),
	RightPanel(right_panel::Message),
	HomePage(pages::home::Message),
}

#[derive(Debug, Default, Clone)]
pub struct State {
	menu: menu::Menu,
	content: right_panel::RightPanel,

	home_page: pages::home::Home,
}

impl State {
	pub fn new() -> (Self, Task<Message>) {
		(Self::default(), Task::done(Message::FirstRun))
	}

	pub fn view(&self) -> Element<Message> {
		let page: Element<Message> = match self.menu.current_page {
			menu::Page::Home => self.home_page.view().map(Message::HomePage),
			menu::Page::Mods => Column::new()
				.spacing(10.)
				// .push(Text::new("Mods").size(30))
				.push(Text::new("Here you can manage your installed mods.").size(20))
				.into(),
			menu::Page::Settings => Column::new()
				.spacing(10.)
				// .push(Text::new("Settings").size(30))
				.push(Text::new("Here you can configure the toolkit.").size(20))
				.into(),
		};

		Container::new(
			Column::new()
				.spacing(10.)
				.push(
					Row::new()
						.push(Text::new("Exanima Modding Toolkit").size(30))
						.width(iced::Length::Fill),
				)
				.push(Rule::horizontal(1.))
				.push(
					Column::new()
						.push(self.menu.view().map(Message::Menu))
						.push(page),
				),
		)
		.padding(Padding::new(12.0))
		.into()
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::FirstRun => Task::done(pages::home::Message::default()).map(Message::HomePage),
			Message::EventOccurred(event) => {
				if let Event::Window(window::Event::CloseRequested) = event {
					window::get_latest().and_then(window::close)
				} else {
					Task::none()
				}
			}
			Message::HomePage(message) => self.home_page.update(message),
			Message::Menu(message) => self.menu.update(message),
			_ => Task::none(),
		}
	}

	pub fn subscription(&self) -> Subscription<Message> {
		event::listen().map(Message::EventOccurred)
	}
}

fn theme(_state: &State) -> Theme {
	Theme::CatppuccinMocha
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
