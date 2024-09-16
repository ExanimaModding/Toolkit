pub mod list;
pub mod modal;

use iced::{
	widget::{self, container, text},
	Element,
};

pub fn tooltip<'a, Message, Theme, Renderer>(
	content: impl Into<Element<'a, Message, Theme, Renderer>>,
	tooltip: &'a str,
) -> widget::tooltip::Tooltip<'a, Message, Theme, Renderer>
where
	Theme: container::Catalog + text::Catalog + 'a,
	Renderer: iced_core::text::Renderer + 'a,
{
	widget::tooltip(
		content,
		text(tooltip).size(14),
		widget::tooltip::Position::Top,
	)
	.padding(8)
}
