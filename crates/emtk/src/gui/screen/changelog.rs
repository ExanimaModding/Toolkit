use std::time::Instant;

use iced::{
	theme,
	widget::{container, markdown, scrollable, text, Column},
	Background, Border, Element, Padding, Size, Task, Theme,
};
use lilt::{Animated, Easing};

use crate::gui::{GetLatestReleaseState, FADE_DURATION};

pub enum Action {
	LinkClicked(String),
	None,
}

#[derive(Debug, Clone)]
pub struct Changelog {
	pub content: Vec<markdown::Item>,
	fade: Animated<bool, Instant>,
	pub latest_release: GetLatestReleaseState,
	pub size: Option<Size>,
	theme: Theme,
}

#[derive(Debug, Clone)]
pub enum Message {
	FadeOut,
	LinkClicked(String),
	SizeChanged(Size),
}

impl Changelog {
	pub fn new(
		content: Vec<markdown::Item>,
		// TODO: state doesn't get updated in this context
		latest_release: GetLatestReleaseState,
		size: Option<Size>,
		theme: Theme,
	) -> Self {
		let now = Instant::now();

		Self {
			content,
			fade: Animated::new(false)
				.duration(FADE_DURATION as f32)
				.easing(Easing::EaseOut)
				.delay(0.)
				.auto_start(true, now),
			latest_release,
			size,
			theme,
		}
	}

	pub fn update(&mut self, message: Message) -> (Task<Message>, Action) {
		let now = Instant::now();

		match message {
			Message::FadeOut => self.fade.transition(false, now),
			Message::LinkClicked(url) => return (Task::none(), Action::LinkClicked(url)),
			Message::SizeChanged(size) => self.size = Some(size),
		}

		(Task::none(), Action::None)
	}

	// FIX: animations don't affect emojis in markdown
	pub fn view(&self) -> Element<Message> {
		let now = Instant::now();

		let loading_con = container(text("Checking for updates..."))
			.padding(12)
			.style(move |theme: &Theme| {
				let palette = theme.palette();
				let animate_alpha = self.fade.animate_bool(0., 1., now);

				container::Style::default()
					.color(palette.text.scale_alpha(animate_alpha))
					.background(palette.background.scale_alpha(animate_alpha))
					.border(Border::default().rounded(8))
			});

		match &self.latest_release {
			GetLatestReleaseState::NotStarted => loading_con.into(),
			GetLatestReleaseState::Loading => loading_con.into(),
			GetLatestReleaseState::Loaded(_) => {
				let animate_alpha = self.fade.animate_bool(0., 1., now);
				let con = container(
					Column::new()
						.push(
							scrollable(
								markdown(&self.content, markdown::Settings::default(), {
									let mut style =
										markdown::Style::from_palette(self.theme.palette());
									style.inline_code_highlight.background = style
										.inline_code_highlight
										.background
										.scale_alpha(animate_alpha);
									style.inline_code_color =
										style.inline_code_color.scale_alpha(animate_alpha);
									style.link_color = style.link_color.scale_alpha(animate_alpha);
									style
								})
								.map(|url| Message::LinkClicked(url.to_string())),
							)
							.style(move |theme, status| {
								let mut style = scrollable::default(theme, status);
								if let Some(background) = style.vertical_rail.background {
									style.vertical_rail.background =
										Some(background.scale_alpha(animate_alpha));
								};
								style.vertical_rail.scroller.color = style
									.vertical_rail
									.scroller
									.color
									.scale_alpha(animate_alpha);
								style
							}),
						)
						.spacing(10.),
				)
				.padding(12)
				.style(move |theme| {
					let palette = theme.palette();

					container::Style::default()
						.color(palette.text.scale_alpha(animate_alpha))
						.background(palette.background.scale_alpha(animate_alpha))
						.border(Border::default().rounded(8))
				});

				if let Some(size) = self.size {
					con.width(size.width).height(size.height)
				} else {
					con
				}
			}
			.into(),
			GetLatestReleaseState::Error(error) => text(format!("Error: {}", error)).into(),
		}
	}
}
