pub mod list;
pub mod modal;

use iced::{
	widget::{self, container, text},
	Alignment, Element, Length,
};

use crate::gui::theme;

pub fn context_menu<'a, Message>(
	content: impl Into<Element<'a, Message>>,
) -> widget::Container<'a, Message>
where
	Message: Clone + 'a,
{
	container(content)
		.padding(6)
		.width(Length::Fixed(164.))
		.style(theme::context_menu)
}

pub fn icon<'a>(handle: impl Into<widget::svg::Handle>) -> widget::svg::Svg<'a> {
	widget::svg(handle)
		.width(Length::Shrink)
		.height(Length::Fixed(16.))
		.style(theme::svg)
}

pub fn tooltip<'a, Message>(
	content: impl Into<Element<'a, Message>>,
	tooltip: &'a str,
	alpha: f32,
) -> widget::tooltip::Tooltip<'a, Message> {
	widget::tooltip(
		content,
		text(tooltip).size(14),
		widget::tooltip::Position::Top,
	)
	.padding(8)
	.style(move |theme| theme::tooltip(theme, alpha))
}
