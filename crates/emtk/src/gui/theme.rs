use std::time::Instant;

use iced::{
	widget::{self, container},
	Border, Color, Shadow, Theme, Vector,
};
use lilt::Animated;

pub fn button(theme: &Theme, status: widget::button::Status) -> widget::button::Style {
	let mut style = widget::button::primary(theme, status);
	style.border = Border::default().rounded(8);
	style
}

pub fn svg(theme: &Theme, _status: widget::svg::Status) -> widget::svg::Style {
	widget::svg::Style {
		color: Some(theme.palette().text),
	}
}

pub fn svg_button(theme: &Theme, _status: widget::svg::Status) -> widget::svg::Style {
	widget::svg::Style {
		color: Some(if theme.extended_palette().is_dark {
			Color::BLACK
		} else {
			Color::WHITE
		}),
	}
}

pub fn tooltip(theme: &Theme, fade: &Animated<bool, Instant>, now: Instant) -> container::Style {
	let palette = theme.extended_palette();
	let animate_alpha = fade.animate_bool(0., 1., now);

	container::Style::default()
		.background(palette.background.base.color.scale_alpha(animate_alpha))
		.border(
			Border::default()
				.color(palette.background.weak.color.scale_alpha(animate_alpha))
				.width(1.)
				.rounded(12),
		)
		.shadow(Shadow {
			color: Color::BLACK.scale_alpha(animate_alpha),
			offset: Vector::new(2., 2.),
			blur_radius: 8.,
		})
}

pub fn transparent_button(theme: &Theme, status: widget::button::Status) -> widget::button::Style {
	let palette = theme.palette();
	let extended_palette = theme.extended_palette();

	let (background, text) = match status {
		widget::button::Status::Hovered => (extended_palette.background.weak.color, palette.text),
		widget::button::Status::Disabled => (
			extended_palette.primary.strong.color,
			if extended_palette.is_dark {
				Color::BLACK
			} else {
				Color::WHITE
			},
		),
		_ => (Color::TRANSPARENT, palette.text),
	};

	let mut style = widget::button::primary(theme, status).with_background(background);
	style.text_color = text;
	style
}
