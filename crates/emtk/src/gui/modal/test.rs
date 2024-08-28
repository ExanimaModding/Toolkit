use super::Message;
use iced::{
	widget::{container, text, Column},
	Element,
};

pub fn view<'a>() -> Element<'a, Message> {
	container(Column::new().push(text("This is a test modal!"))).into()
}
