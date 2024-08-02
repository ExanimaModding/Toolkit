use iced::{
	widget::{self, Button, Row, Text},
	Element, Task, Theme,
};

#[derive(Debug, Clone, Copy)]
pub enum Message {
	PageChange(Page),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Page {
	#[default]
	Home,
	Changelog,
	Mods,
	Settings,
}

impl std::fmt::Display for Page {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Menu {
	pub current_page: Page,
}

impl Menu {
	pub fn view(&self) -> Element<'_, Message> {
		let mut column: Row<Message> = Row::new().width(iced::Length::Fill).spacing(10.);

		for item in &[Page::Home, Page::Changelog, Page::Mods, Page::Settings] {
			let button: Button<Message> =
				Button::new(Text::new(item.to_string()).center().size(20.))
					.on_press(Message::PageChange(*item))
					.style(|theme: &Theme, status| {
						let palette = theme.extended_palette();
						match self.current_page == *item {
							true => widget::button::Style::default()
								.with_background(palette.primary.strong.color),
							false => widget::button::primary(theme, status)
								.with_background(palette.primary.base.color),
						}
					});
			// .width(100.);

			column = column.push(button);
		}

		column.into()
	}

	pub fn update(
		&mut self,
		_app_state: &mut crate::gui::state::AppState,
		message: Message,
	) -> Task<crate::gui::Message> {
		match message {
			Message::PageChange(page) => {
				self.current_page = page;
				Task::none()
			}
		}
	}
}
