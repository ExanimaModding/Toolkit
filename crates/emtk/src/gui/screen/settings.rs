use iced::{
	widget::{text, Column},
	Element, Task,
};

#[derive(Debug, Default, Clone)]
pub struct Settings {}

#[derive(Debug, Clone)]
pub enum Message {}

impl Settings {
	pub fn update(
		&mut self,
		message: Message,
		_app_state: &mut crate::gui::state::AppState,
	) -> Task<Message> {
		match message {}
	}

	pub fn view(&self) -> Element<Message> {
		// let col = Column::new().push(self.version());

		Column::new().push(text("Settings")).into()
	}
}
