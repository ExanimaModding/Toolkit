use iced::{
	widget::{self, container},
	Border, Color, Shadow, Theme, Vector,
};

pub fn checkbox(
	theme: &Theme,
	status: widget::checkbox::Status,
	scale_alpha: f32,
) -> widget::checkbox::Style {
	let style = widget::checkbox::primary(theme, status);
	widget::checkbox::Style {
		background: style.background.scale_alpha(scale_alpha),
		icon_color: style.icon_color.scale_alpha(scale_alpha),
		border: Border {
			color: style.border.color.scale_alpha(scale_alpha),
			width: style.border.width,
			radius: style.border.radius,
		},
		text_color: style.text_color.map(|color| color.scale_alpha(scale_alpha)),
	}
}

pub fn rule(theme: &Theme, scale_alpha: f32) -> widget::rule::Style {
	let mut style = widget::rule::default(theme);
	style.color = style.color.scale_alpha(scale_alpha);
	style
}

pub fn pick_list(
	theme: &Theme,
	status: widget::pick_list::Status,
	scale_alpha: f32,
) -> widget::pick_list::Style {
	let style = widget::pick_list::default(theme, status);
	widget::pick_list::Style {
		text_color: style.text_color.scale_alpha(scale_alpha),
		placeholder_color: style.placeholder_color.scale_alpha(scale_alpha),
		handle_color: style.handle_color.scale_alpha(scale_alpha),
		background: style.background.scale_alpha(scale_alpha),
		border: Border {
			color: style.border.color.scale_alpha(scale_alpha),
			width: style.border.width,
			radius: style.border.radius,
		},
	}
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

pub fn svg_danger(theme: &Theme, _status: widget::svg::Status) -> widget::svg::Style {
	widget::svg::Style {
		color: Some(theme.palette().danger),
	}
}

pub fn tooltip(theme: &Theme, scale_alpha: f32) -> container::Style {
	let palette = theme.extended_palette();

	container::Style::default()
		.color(theme.palette().text.scale_alpha(scale_alpha))
		.background(palette.background.base.color.scale_alpha(scale_alpha))
		.border(
			Border::default()
				.color(palette.background.weak.color.scale_alpha(scale_alpha))
				.width(1.)
				.rounded(12),
		)
		.shadow(Shadow {
			color: Color::BLACK.scale_alpha(scale_alpha),
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
