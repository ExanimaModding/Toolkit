use iced::{
	widget::{self, button, text, Row},
	Element, Task, Theme,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Page {
	#[default]
	Home,
	Settings,
}

impl std::fmt::Display for Page {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
	PageChange(Page),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Menu {
	pub current_page: Page,
}

impl Menu {
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

	pub fn view(&self) -> Element<'_, Message> {
		let mut column: Row<Message> = Row::new().width(iced::Length::Fill).spacing(10.);

		for item in &[Page::Home, Page::Settings] {
			let button = button(text(item.to_string()).center().size(20.))
				.on_press(Message::PageChange(*item))
				.style(|theme: &Theme, status| {
					let palette = theme.extended_palette();
					match self.current_page == *item {
						// FIX: affects all buttons on the page
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
}
