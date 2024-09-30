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

pub fn context_menu(theme: &Theme) -> widget::container::Style {
	let palette = theme.extended_palette();
	container::Style::default()
		.background(palette.background.base.color)
		.border(
			Border::default()
				.color(palette.background.weak.color)
				.width(1)
				.rounded(3),
		)
		.shadow(Shadow {
			color: Color::BLACK,
			offset: Vector::new(2., 2.),
			blur_radius: 8.,
		})
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
		color: Some(theme.extended_palette().background.base.text),
	}
}

pub fn svg_button(theme: &Theme, _status: widget::svg::Status) -> widget::svg::Style {
	widget::svg::Style {
		color: Some(theme.extended_palette().primary.strong.text),
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
		.color(
			theme
				.extended_palette()
				.background
				.base
				.text
				.scale_alpha(scale_alpha),
		)
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
	let palette = theme.extended_palette();

	let (background, text) = match status {
		widget::button::Status::Hovered => {
			(palette.background.weak.color, palette.background.weak.text)
		}
		widget::button::Status::Disabled => {
			(palette.primary.strong.color, palette.primary.strong.text)
		}
		_ => (Color::TRANSPARENT, palette.background.base.text),
	};

	let mut style = widget::button::primary(theme, status).with_background(background);
	style.text_color = text;
	style
}

pub fn transparent_danger_button(
	theme: &Theme,
	status: widget::button::Status,
) -> widget::button::Style {
	let palette = theme.extended_palette();

	let (background, text) = match status {
		widget::button::Status::Hovered => {
			(palette.danger.strong.color, palette.danger.strong.text)
		}
		widget::button::Status::Disabled => (
			palette.background.base.color.scale_alpha(0.5),
			palette.danger.strong.color.scale_alpha(0.5),
		),
		_ => (Color::TRANSPARENT, palette.danger.strong.color),
	};

	let mut style = widget::button::danger(theme, status).with_background(background);
	style.text_color = text;
	style
}

pub fn transparent_primary_button(
	theme: &Theme,
	status: widget::button::Status,
) -> widget::button::Style {
	let palette = theme.extended_palette();

	let (background, text) = match status {
		widget::button::Status::Hovered => {
			(palette.primary.strong.color, palette.primary.strong.text)
		}
		widget::button::Status::Disabled => (
			palette.background.base.color.scale_alpha(0.5),
			palette.background.base.text.scale_alpha(0.5),
		),
		_ => (Color::TRANSPARENT, palette.background.base.text),
	};

	let mut style = widget::button::primary(theme, status).with_background(background);
	style.text_color = text;
	style
}
