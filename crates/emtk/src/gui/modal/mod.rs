use crate::gui::Message;
use iced::{
	widget::{center, container, mouse_area, opaque, stack},
	Color, Element,
};

pub mod launching;
pub mod test;

#[derive(Clone, Debug)]
pub enum ModalKind {
	Launching,
	Test,
}

impl ModalKind {
	pub fn view(&self) -> Element<Message> {
		match self {
			ModalKind::Launching => launching::view(),
			ModalKind::Test => test::view(),
		}
	}
}

pub fn modal<'a, Message>(
	base: impl Into<Element<'a, Message>>,
	content: impl Into<Element<'a, Message>>,
	on_blur: Message,
) -> Element<'a, Message>
where
	Message: Clone + 'a,
{
	stack![
		base.into(),
		opaque(
			mouse_area(center(opaque(content)).style(|_theme| {
				container::Style {
					background: Some(
						Color {
							a: 0.8,
							..Color::BLACK
						}
						.into(),
					),
					..container::Style::default()
				}
			}))
			.on_press(on_blur)
		)
	]
	.into()
}
