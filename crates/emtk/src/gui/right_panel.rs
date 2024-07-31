use iced::{
	widget::{Column, Text},
	Element, Task,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct RightPanel {
	// state: State,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
	None,
}

impl RightPanel {
	pub fn new() -> Self {
		Self {
			// state: State::default(),
		}
	}

	pub fn view(&self) -> Element<'_, Message> {
		Column::new()
			.width(iced::Length::Fill)
			.push(Text::new("Right Pane"))
			// .push(Button::new(&mut self.state, Text::new("Click me!")).on_press(Message::RightPane))
			.into()
	}

	pub fn update(&mut self, message: Message) -> Task<crate::gui::Message> {
		match message {
			// Message::RightPane => Task::none(),
			_ => Task::none(),
		}
	}
}
