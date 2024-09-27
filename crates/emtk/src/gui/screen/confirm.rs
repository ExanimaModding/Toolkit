use std::time::Instant;

use iced::{
	widget::{button, container, horizontal_space, text, Column, Row},
	Border, Element, Size, Theme,
};
use lilt::{Animated, Easing};

use crate::gui::constants::FADE_DURATION;

#[derive(Debug, Clone)]
pub enum Action {
	ModDeleted(usize),
	None,
}

#[derive(Debug, Clone)]
pub struct Confirm {
	action: Action,
	fade: Animated<bool, Instant>,
	size: Option<Size>,
}

#[derive(Debug, Clone)]
pub enum Message {
	Canceled,
	Confirmed,
	FadeOut,
	SizeChanged(Size),
}

impl Confirm {
	pub fn new(action: Action, size: Option<Size>) -> Self {
		let now = Instant::now();

		Self {
			action,
			fade: Animated::new(false)
				.duration(FADE_DURATION as f32)
				.easing(Easing::EaseOut)
				.delay(0.)
				.auto_start(true, now),
			size,
		}
	}

	pub fn update(&mut self, message: Message) -> Action {
		let now = Instant::now();

		match message {
			Message::Canceled => (),
			Message::Confirmed => return self.action.clone(),
			Message::FadeOut => self.fade.transition(false, now),
			Message::SizeChanged(size) => self.size = Some(size),
		}

		Action::None
	}

	pub fn view(&self) -> Element<Message> {
		let now = Instant::now();
		let animate_alpha = self.fade.animate_bool(0., 1., now);

		let con = container(
			Column::new()
				// TODO: provide more detailed information in dialog
				.push(text("Are you sure?"))
				.push(
					Row::new()
						.push(horizontal_space())
						.push(button(text("Cancel")).on_press(Message::Canceled).style(
							move |theme, status| {
								let mut style = button::primary(theme, status);
								if let Some(background) = style.background {
									style.background = Some(background.scale_alpha(animate_alpha))
								}
								style.text_color = style.text_color.scale_alpha(animate_alpha);
								style
							},
						))
						.push(button(text("Confirm")).on_press(Message::Confirmed).style(
							move |theme, status| {
								let mut style = button::danger(theme, status);
								if let Some(background) = style.background {
									style.background = Some(background.scale_alpha(animate_alpha))
								}
								style.text_color = style.text_color.scale_alpha(animate_alpha);
								style
							},
						))
						.spacing(6),
				)
				.spacing(12),
		)
		.padding(12)
		.style(move |theme: &Theme| {
			let palette = theme.palette();

			container::Style::default()
				.color(palette.text.scale_alpha(animate_alpha))
				.background(palette.background.scale_alpha(animate_alpha))
				.border(Border::default().rounded(8))
		});

		if let Some(size) = self.size {
			con.width(size.width)
		} else {
			con
		}
		.into()
	}
}
