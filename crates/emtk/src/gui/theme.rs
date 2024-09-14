use iced::{widget, Border, Color, Theme};

pub fn button(theme: &Theme, status: widget::button::Status) -> widget::button::Style {
	let mut style = widget::button::primary(theme, status);
	style.border = Border::default().rounded(8);
	style
}

pub fn transparent_button(theme: &Theme, status: widget::button::Status) -> widget::button::Style {
	let palette = theme.palette();
	let extended_palette = theme.extended_palette();

	let (background, text) = match status {
		widget::button::Status::Hovered => (extended_palette.background.weak.color, palette.text),
		widget::button::Status::Disabled => (extended_palette.primary.strong.color, Color::BLACK),
		_ => (Color::TRANSPARENT, palette.text),
	};

	let mut style = widget::button::primary(theme, status).with_background(background);
	style.text_color = text;
	style.border = Border::default().rounded(8);
	style
}