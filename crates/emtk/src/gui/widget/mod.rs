//! Override iced's built-in widgets or use custom widgets

// pub mod selectable;
pub mod toast;

use iced::{
	advanced,
	widget::{
		button as iced_button, container as iced_container, row as iced_row,
		scrollable as iced_scrollable, text as iced_text, tooltip as iced_tooltip, Button,
		Container, Row, Scrollable, Text, Tooltip,
	},
	Border, Color, Element, Font, Padding, Shadow, Theme, Vector,
};

pub mod button {
	pub use iced_button::Style;

	use super::*;

	fn default(text_color: Color) -> iced_button::Style {
		iced_button::Style {
			text_color,
			border: Border::default().rounded(3),
			..Default::default()
		}
	}

	pub fn danger(theme: &Theme, status: iced_button::Status) -> iced_button::Style {
		let ext_palette = theme.extended_palette();
		let default = default(ext_palette.background.base.text);
		match status {
			iced_button::Status::Active => default,
			iced_button::Status::Hovered => iced_button::Style {
				background: Some(ext_palette.danger.weak.color.into()),
				text_color: ext_palette.danger.weak.text,
				..default
			},
			iced_button::Status::Pressed => iced_button::Style {
				background: Some(ext_palette.danger.base.color.into()),
				text_color: ext_palette.danger.base.text,
				..default
			},
			iced_button::Status::Disabled => iced_button::Style {
				text_color: default.text_color.scale_alpha(0.5),
				..default
			},
		}
	}

	pub fn primary(theme: &Theme, status: iced_button::Status) -> iced_button::Style {
		let ext_palette = theme.extended_palette();
		let default = default(ext_palette.background.base.text);
		match status {
			iced_button::Status::Active => default,
			iced_button::Status::Hovered => iced_button::Style {
				background: Some(ext_palette.primary.weak.color.into()),
				text_color: ext_palette.primary.weak.text,
				..default
			},
			iced_button::Status::Pressed => iced_button::Style {
				background: Some(ext_palette.primary.base.color.into()),
				text_color: ext_palette.primary.base.text,
				..default
			},
			iced_button::Status::Disabled => iced_button::Style {
				text_color: default.text_color.scale_alpha(0.5),
				..default
			},
		}
	}

	pub fn success(theme: &Theme, status: iced_button::Status) -> iced_button::Style {
		let ext_palette = theme.extended_palette();
		let default = default(ext_palette.background.base.text);
		match status {
			iced_button::Status::Active => default,
			iced_button::Status::Hovered => iced_button::Style {
				background: Some(ext_palette.success.weak.color.into()),
				text_color: ext_palette.success.weak.text,
				..default
			},
			iced_button::Status::Pressed => iced_button::Style {
				background: Some(ext_palette.success.base.color.into()),
				text_color: ext_palette.success.base.text,
				..default
			},
			iced_button::Status::Disabled => iced_button::Style {
				text_color: default.text_color.scale_alpha(0.5),
				..default
			},
		}
	}
}

pub fn button<'a, Message>(content: impl Into<Element<'a, Message>>) -> Button<'a, Message> {
	iced_button(content).style(button::primary)
}

pub mod container {
	use super::*;

	pub use iced::widget::container::Style;

	pub fn bordered_box(theme: &Theme) -> iced_container::Style {
		let bordered_box = iced_container::bordered_box(theme);
		iced_container::Style {
			shadow: Shadow {
				color: Color::BLACK.scale_alpha(0.8),
				offset: Vector::new(3., 3.),
				blur_radius: 8.,
			},
			..bordered_box
		}
	}
}

pub fn container<'a, Message, Theme, Renderer>(
	content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
	Theme: iced_container::Catalog + 'a,
	Renderer: advanced::Renderer,
{
	iced_container(content)
}

pub fn close_button<'a, Message>() -> Button<'a, Message> {
	let size = 20;
	iced_button(icon::close().size(8).center())
		.padding(4)
		.width(size)
		.height(size)
		.style(|theme: &Theme, status| {
			let ext_palette = theme.extended_palette();
			let danger = button::danger(theme, status);
			if let iced_button::Status::Active = status {
				iced_button::Style {
					background: Some(ext_palette.primary.weak.color.into()),
					text_color: ext_palette.primary.weak.text,
					..danger
				}
			} else {
				danger
			}
		})
}

pub fn default_value<'a, Message: 'a + Clone>(
	content: Element<'a, Message>,
	on_press: Message,
) -> Row<'a, Message> {
	iced_row![
		content,
		tooltip(
			button(icon::rotate_ccw())
				.on_press(on_press)
				.style(button::danger),
			iced_text("Reset to default value"),
			tooltip::Position::Right
		)
	]
	.spacing(3)
}

pub mod icon {
	use super::*;

	/// Helper used for accessing Lucide icons in the app
	fn lucide<'a>(codepoint: char) -> Text<'a> {
		iced_text(codepoint).font(Font::with_name("lucide"))
	}

	/// Unicode for this app's Lucide rotate x icon
	pub fn close<'a>() -> Text<'a> {
		lucide('\u{E802}')
	}

	/// Unicode for this app's Lucide check icon
	pub fn check<'a>() -> Text<'a> {
		lucide('\u{E805}')
	}

	/// Unicode for this app's Lucide circle-check icon
	pub fn circle_check<'a>() -> Text<'a> {
		lucide('\u{E812}')
	}

	/// Unicode for this app's Lucide circle-x icon
	pub fn circle_x<'a>() -> Text<'a> {
		lucide('\u{E80F}')
	}

	/// Unicode for this app's Lucide folder icon
	pub fn folder<'a>() -> Text<'a> {
		lucide('\u{E80E}')
	}

	/// Unicode for this app's Lucide folders icon
	pub fn folders<'a>() -> Text<'a> {
		lucide('\u{E80C}')
	}

	/// Unicode for this app's Lucide folder-open icon
	pub fn folder_open<'a>() -> Text<'a> {
		lucide('\u{E80D}')
	}

	/// Unicode for this app's Lucide info icon
	pub fn info<'a>() -> Text<'a> {
		lucide('\u{E807}')
	}

	/// Unicode for this app's Lucide layers icon
	pub fn layers<'a>() -> Text<'a> {
		lucide('\u{E80B}')
	}

	/// Unicode for this app's Lucide logs icon
	pub fn logs<'a>() -> Text<'a> {
		lucide('\u{E80A}')
	}

	/// Unicode for this app's Lucide rotate menu icon
	pub fn menu<'a>() -> Text<'a> {
		lucide('\u{E800}')
	}

	/// Unicode for this app's Lucide octagon-x icon
	pub fn octagon_x<'a>() -> Text<'a> {
		lucide('\u{E811}')
	}

	/// Unicode for this app's Lucide rotate play icon
	pub fn play<'a>() -> Text<'a> {
		lucide('\u{E801}')
	}

	/// Unicode for this app's Lucide plus icon
	pub fn plus<'a>() -> Text<'a> {
		lucide('\u{E804}')
	}

	/// Unicode for this app's Lucide rotate clockwise icon typically used for
	/// refresh indication
	pub fn rotate_cw<'a>() -> Text<'a> {
		lucide('\u{E808}')
	}

	/// Unicode for this app's Lucide rotate counter-clockwise icon typically used
	/// for reset indication
	pub fn rotate_ccw<'a>() -> Text<'a> {
		lucide('\u{E803}')
	}

	/// Unicode for this app's Lucide settings icon
	pub fn settings<'a>() -> Text<'a> {
		lucide('\u{E809}')
	}

	/// Unicode for this app's Lucide square-arrow-out-up-right icon typically used
	/// for external link indication
	pub fn square_arrow_out_up_right<'a>() -> Text<'a> {
		lucide('\u{E813}')
	}

	/// Unicode for this app's Lucide trach-2 icon
	pub fn trash<'a>() -> Text<'a> {
		lucide('\u{E806}')
	}

	/// Unicode for this app's Lucide triangle-alert icon
	pub fn triangle_alert<'a>() -> Text<'a> {
		lucide('\u{E810}')
	}
}

pub mod scrollable {
	pub use iced::widget::scrollable::{scroll_to, AbsoluteOffset, Direction, Id, Scrollbar};
}

pub fn scrollable<'a, Message: 'a>(
	content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
	iced_scrollable(iced_container(content).padding(Padding::default().bottom(10).right(10)))
		.direction(iced_scrollable::Direction::Both {
			vertical: iced_scrollable::Scrollbar::default(),
			horizontal: iced_scrollable::Scrollbar::default(),
		})
}

pub mod text {
	use super::*;

	pub use iced::widget::text::{danger, primary, success};

	pub fn warning(theme: &Theme) -> iced_text::Style {
		iced_text::Style {
			color: Some(theme.palette().warning),
		}
	}
}

pub fn text<'a>(text: impl iced_text::IntoFragment<'a>) -> Text<'a> {
	iced_text(text)
}

pub mod tooltip {
	pub use iced::widget::tooltip::Position;
}

pub fn tooltip<'a, Message: 'a>(
	content: impl Into<Element<'a, Message>>,
	tooltip: impl Into<Element<'a, Message>>,
	position: iced_tooltip::Position,
) -> Tooltip<'a, Message> {
	iced_tooltip(
		content,
		iced_container(tooltip)
			.padding(4)
			.style(container::bordered_box),
		position,
	)
}

// pub struct Logs;

// impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Logs
// where
// 	Renderer: advanced::Renderer,
// {
// 	fn size(&self) -> Size<Length> {
// 		Size::new(Length::Fill, Length::Fill)
// 	}

// 	fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
// 		Node::new(Size::new(0., 0.))
// 	}

// 	fn draw(
// 		&self,
// 		tree: &Tree,
// 		renderer: &mut Renderer,
// 		theme: &Theme,
// 		style: &renderer::Style,
// 		layout: Layout<'_>,
// 		cursor: mouse::Cursor,
// 		viewport: &Rectangle,
// 	) {
// 	}
// }

// impl<Message, Theme, Renderer> From<Logs> for Element<'_, Message, Theme, Renderer>
// where
// 	Renderer: advanced::Renderer,
// {
// 	fn from(widget: Logs) -> Self {
// 		Self::new(widget)
// 	}
// }

// use iced::{
// 	widget::{self, TextInput},
// 	Element,
// };

// use crate::gui::style::text_input;

// pub struct Logs<'a, Message>
// where
// 	Message: Clone,
// {
// 	widget: TextInput<'a, Message>,
// }

// impl<Message> Default for Logs<'_, Message>
// where
// 	Message: Clone,
// {
// 	fn default() -> Self {
// 		Self {
// 			widget: widget::text_input("no logs here...", "").style(text_input::invisible),
// 		}
// 	}
// }

// impl<'a, Message> From<TextInput<'a, Message>> for Logs<'a, Message>
// where
// 	Message: Clone,
// {
// 	fn from(widget: TextInput<'a, Message>) -> Self {
// 		Self { widget }
// 	}
// }

// impl<'a, Message> From<Logs<'a, Message>> for Element<'a, Message>
// where
// 	Message: Clone + 'a,
// {
// 	fn from(logs: Logs<'a, Message>) -> Self {
// 		logs.widget.into()
// 	}
// }

// pub(super) fn logs<'a, Message>() -> Logs<'a, Message>
// where
// 	Message: Clone,
// {
// 	Logs::default()
// }
// 	Logs::default()
// }
// 	Logs::default()
// }
