use super::*;

// TODO: Bindings
// - Unfocus
// - Copy
// - Select(Motion)
// - SelectWord
// - SelectLine
// - SelectAll
// TODO: Status
// - Hovered
pub struct Selectable {
	editor: cosmic_text::Editor<'static>,
}

impl Selectable {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for Selectable {
	fn default() -> Self {
		Self {
			editor: cosmic_text::Editor::new(cosmic_text::Buffer::new_empty(
				cosmic_text::Metrics::new(1., 1.),
			)),
		}
	}
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Selectable
where
	Renderer: advanced::Renderer,
{
	fn size(&self) -> Size<Length> {
		Size::new(Length::Fill, Length::Fill)
	}

	fn layout(
		&self,
		tree: &mut widget::Tree,
		renderer: &Renderer,
		limits: &layout::Limits,
	) -> layout::Node {
		layout::Node::new(Size::default())
	}

	fn draw(
		&self,
		tree: &widget::Tree,
		renderer: &mut Renderer,
		theme: &Theme,
		style: &renderer::Style,
		layout: Layout<'_>,
		_cursor: mouse::Cursor,
		viewport: &Rectangle,
	) {
	}
}

impl<Message, Theme, Renderer> From<Selectable> for Element<'_, Message, Theme, Renderer>
where
	Renderer: advanced::Renderer,
{
	fn from(widget: Selectable) -> Self {
		Self::new(widget)
	}
}
