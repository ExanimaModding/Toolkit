//! License SPDX: GPL-3.0-only
//! Source: https://github.com/squidowl/halloy/blob/main/src/widget/modal.rs

use std::time::Instant;

use iced::{
	advanced::{
		self,
		layout::{self, Layout},
		overlay, renderer,
		widget::{self, Widget},
		Clipboard, Shell,
	},
	alignment::Alignment,
	event,
	keyboard::{self, key},
	mouse, Color, Element, Event, Length, Point, Rectangle, Size, Vector,
};
use lilt::Animated;

pub fn modal<'a, Message, Theme, Renderer>(
	animated: Animated<bool, Instant>,
	base: impl Into<Element<'a, Message, Theme, Renderer>>,
	modal: impl Into<Element<'a, Message, Theme, Renderer>>,
	on_blur: impl Fn() -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
	Theme: 'a,
	Renderer: 'a + advanced::Renderer,
	Message: 'a,
{
	Modal::new(animated, base, modal, on_blur).into()
}

/// A widget that centers a modal element over some base element
pub struct Modal<'a, Message, Theme, Renderer> {
	animation: Animated<bool, Instant>,
	base: Element<'a, Message, Theme, Renderer>,
	modal: Element<'a, Message, Theme, Renderer>,
	on_blur: Box<dyn Fn() -> Message + 'a>,
}

impl<'a, Message, Theme, Renderer> Modal<'a, Message, Theme, Renderer> {
	/// Returns a new [`Modal`]
	pub fn new(
		animation: Animated<bool, Instant>,
		base: impl Into<Element<'a, Message, Theme, Renderer>>,
		modal: impl Into<Element<'a, Message, Theme, Renderer>>,
		on_blur: impl Fn() -> Message + 'a,
	) -> Self {
		Self {
			animation,
			base: base.into(),
			modal: modal.into(),
			on_blur: Box::new(on_blur),
		}
	}
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
	for Modal<'a, Message, Theme, Renderer>
where
	Renderer: advanced::Renderer,
{
	fn children(&self) -> Vec<widget::Tree> {
		vec![
			widget::Tree::new(&self.base),
			widget::Tree::new(&self.modal),
		]
	}

	fn diff(&self, tree: &mut widget::Tree) {
		tree.diff_children(&[&self.base, &self.modal]);
	}

	fn size(&self) -> Size<Length> {
		self.base.as_widget().size()
	}

	fn layout(
		&self,
		tree: &mut widget::Tree,
		renderer: &Renderer,
		limits: &layout::Limits,
	) -> layout::Node {
		self.base
			.as_widget()
			.layout(&mut tree.children[0], renderer, limits)
	}

	fn on_event(
		&mut self,
		state: &mut widget::Tree,
		event: Event,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		renderer: &Renderer,
		clipboard: &mut dyn Clipboard,
		shell: &mut Shell<'_, Message>,
		viewport: &Rectangle,
	) -> event::Status {
		self.base.as_widget_mut().on_event(
			&mut state.children[0],
			event,
			layout,
			cursor,
			renderer,
			clipboard,
			shell,
			viewport,
		)
	}

	fn draw(
		&self,
		state: &widget::Tree,
		renderer: &mut Renderer,
		theme: &Theme,
		style: &renderer::Style,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		viewport: &Rectangle,
	) {
		self.base.as_widget().draw(
			&state.children[0],
			renderer,
			theme,
			style,
			layout,
			cursor,
			viewport,
		);
	}

	fn overlay<'b>(
		&'b mut self,
		state: &'b mut widget::Tree,
		layout: Layout<'_>,
		_renderer: &Renderer,
		translation: Vector,
	) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
		Some(overlay::Element::new(Box::new(Overlay {
			animation: self.animation.clone(),
			position: layout.position() + translation,
			content: &mut self.modal,
			tree: &mut state.children[1],
			size: layout.bounds().size(),
			on_blur: &self.on_blur,
		})))
	}

	fn mouse_interaction(
		&self,
		state: &widget::Tree,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		viewport: &Rectangle,
		renderer: &Renderer,
	) -> mouse::Interaction {
		self.base.as_widget().mouse_interaction(
			&state.children[0],
			layout,
			cursor,
			viewport,
			renderer,
		)
	}

	fn operate(
		&self,
		state: &mut widget::Tree,
		layout: Layout<'_>,
		renderer: &Renderer,
		operation: &mut dyn widget::Operation<()>,
	) {
		self.base
			.as_widget()
			.operate(&mut state.children[0], layout, renderer, operation);
	}
}

struct Overlay<'a, 'b, Message, Theme, Renderer> {
	animation: Animated<bool, Instant>,
	position: Point,
	content: &'b mut Element<'a, Message, Theme, Renderer>,
	tree: &'b mut widget::Tree,
	size: Size,
	on_blur: &'b dyn Fn() -> Message,
}

impl<'a, 'b, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
	for Overlay<'a, 'b, Message, Theme, Renderer>
where
	Renderer: advanced::Renderer,
{
	fn layout(&mut self, renderer: &Renderer, _bounds: Size) -> layout::Node {
		let limits = layout::Limits::new(Size::ZERO, self.size)
			.width(Length::Fill)
			.height(Length::Fill);

		let child = self
			.content
			.as_widget()
			.layout(self.tree, renderer, &limits)
			.align(Alignment::Center, Alignment::Center, limits.max());

		layout::Node::with_children(self.size, vec![child]).move_to(self.position)
	}

	fn on_event(
		&mut self,
		event: Event,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		renderer: &Renderer,
		clipboard: &mut dyn Clipboard,
		shell: &mut Shell<'_, Message>,
	) -> event::Status {
		match event {
			Event::Keyboard(keyboard::Event::KeyPressed {
				key: keyboard::Key::Named(key::Named::Escape),
				..
			}) => {
				shell.publish((self.on_blur)());
				return event::Status::Captured;
			}
			Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
				let bounds = layout.children().next().unwrap().bounds();

				if !cursor.is_over(bounds) {
					shell.publish((self.on_blur)());
					return event::Status::Captured;
				}
			}
			_ => {}
		}

		self.content.as_widget_mut().on_event(
			self.tree,
			event,
			layout.children().next().unwrap(),
			cursor,
			renderer,
			clipboard,
			shell,
			&layout.bounds(),
		)
	}

	fn draw(
		&self,
		renderer: &mut Renderer,
		theme: &Theme,
		style: &renderer::Style,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
	) {
		let now = Instant::now();

		renderer.fill_quad(
			renderer::Quad {
				bounds: layout.bounds(),
				..renderer::Quad::default()
			},
			Color {
				a: self.animation.animate_bool(0.0, 0.8, now),
				..Color::BLACK
			},
		);

		self.content.as_widget().draw(
			self.tree,
			renderer,
			theme,
			style,
			layout.children().next().unwrap(),
			cursor,
			&layout.bounds(),
		);
	}

	fn operate(
		&mut self,
		layout: Layout<'_>,
		renderer: &Renderer,
		operation: &mut dyn widget::Operation<()>,
	) {
		self.content.as_widget().operate(
			self.tree,
			layout.children().next().unwrap(),
			renderer,
			operation,
		);
	}

	fn mouse_interaction(
		&self,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		viewport: &Rectangle,
		renderer: &Renderer,
	) -> mouse::Interaction {
		self.content.as_widget().mouse_interaction(
			self.tree,
			layout.children().next().unwrap(),
			cursor,
			viewport,
			renderer,
		)
	}

	fn overlay<'c>(
		&'c mut self,
		layout: Layout<'_>,
		renderer: &Renderer,
	) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
		self.content.as_widget_mut().overlay(
			self.tree,
			layout.children().next().unwrap(),
			renderer,
			Vector::ZERO,
		)
	}
}

impl<'a, Message, Theme, Renderer> From<Modal<'a, Message, Theme, Renderer>>
	for Element<'a, Message, Theme, Renderer>
where
	Theme: 'a,
	Renderer: 'a + advanced::Renderer,
	Message: 'a,
{
	fn from(modal: Modal<'a, Message, Theme, Renderer>) -> Self {
		Element::new(modal)
	}
}