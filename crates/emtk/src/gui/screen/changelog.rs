use std::time::Instant;

use iced::{
	theme,
	widget::{container, markdown, scrollable, text, Column},
	Background, Border, Element, Padding, Size, Task,
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
		latest_release: GetLatestReleaseState,
		size: Option<Size>,
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
		}
	}

	pub fn update(&mut self, message: Message) -> (Task<Message>, Action) {
		let now = Instant::now();

		match message {
			Message::FadeOut => {
				self.fade.transition(false, now);
				(Task::none(), Action::None)
			}
			Message::LinkClicked(url) => (Task::none(), Action::LinkClicked(url)),
			Message::SizeChanged(size) => {
				self.size = Some(size);
				(Task::none(), Action::None)
			}
		}
	}

	// FIX: animations don't affect emojis in markdown
	pub fn view(&self) -> Element<Message> {
		let now = Instant::now();

		let loading_con = container(text("Checking for updates..."))
			.padding(12)
			.style(move |_theme| {
				let animate_path = self.fade.animate_bool(0., 1., now);
				let palette = theme::Palette::CATPPUCCIN_FRAPPE;
				let mut bg = palette.background;
				bg.a = animate_path;
				container::Style {
					text_color: Some(palette.text),
					background: Some(Background::Color(bg)),
					border: Border {
						radius: 8.0.into(),
						..Default::default()
					},
					..Default::default()
				}
			});

		match &self.latest_release {
			GetLatestReleaseState::NotStarted => loading_con.into(),
			GetLatestReleaseState::Loading => loading_con.into(),
			GetLatestReleaseState::Loaded(_) => {
				let animate_alpha = self.fade.animate_bool(0., 1., now);
				let palette = &theme::palette::EXTENDED_CATPPUCCIN_FRAPPE;
				let mut text = theme::Palette::CATPPUCCIN_FRAPPE.text;
				let mut bg = palette.background.base.color;
				let mut bg_weak = palette.background.weak.color;
				let mut bg_strong = palette.background.strong.color;
				let mut primary = palette.primary.base.color;
				let mut primary_strong = palette.primary.strong.color;
				text.a = animate_alpha;
				bg.a = animate_alpha;
				bg_weak.a = animate_alpha;
				bg_strong.a = animate_alpha;
				primary.a = animate_alpha;
				let con = container(
					Column::new()
						.push(
							scrollable(
								markdown(
									&self.content,
									markdown::Settings::default(),
									markdown::Style {
										inline_code_highlight: markdown::Highlight {
											background: Background::Color(bg),
											border: Border::default(),
										},
										inline_code_padding: Padding::default(),
										inline_code_color: text,
										link_color: primary,
									},
								)
								.map(|url| Message::LinkClicked(url.to_string())),
							)
							.style(move |_theme, status| {
								let (horizontal_scroller, vertical_scroller) = match status {
									scrollable::Status::Active => (bg_strong, bg_strong),
									scrollable::Status::Hovered {
										is_horizontal_scrollbar_hovered,
										is_vertical_scrollbar_hovered,
									} => {
										if is_horizontal_scrollbar_hovered {
											(primary_strong, bg_strong)
										} else if is_vertical_scrollbar_hovered {
											(bg_strong, primary_strong)
										} else {
											(bg_strong, bg_strong)
										}
									}
									scrollable::Status::Dragged {
										is_horizontal_scrollbar_dragged,
										is_vertical_scrollbar_dragged,
									} => {
										if is_horizontal_scrollbar_dragged {
											(primary, bg_strong)
										} else if is_vertical_scrollbar_dragged {
											(bg_strong, primary)
										} else {
											(bg_strong, bg_strong)
										}
									}
								};

								scrollable::Style {
									container: container::Style::default(),
									vertical_rail: scrollable::Rail {
										background: Some(Background::Color(bg_weak)),
										border: Border::default(),
										scroller: scrollable::Scroller {
											color: vertical_scroller,
											border: Border::default(),
										},
									},
									horizontal_rail: scrollable::Rail {
										background: None,
										border: Border::default(),
										scroller: scrollable::Scroller {
											color: horizontal_scroller,
											border: Border::default(),
										},
									},
									gap: None,
								}
							}),
						)
						.spacing(10.),
				)
				.padding(12)
				.style(move |_theme| {
					let animate_alpha = self.fade.animate_bool(0., 1., now);
					let palette = theme::Palette::CATPPUCCIN_FRAPPE;
					let mut text = palette.text;
					let mut bg = palette.background;
					text.a = animate_alpha;
					bg.a = animate_alpha;
					container::Style {
						text_color: Some(text),
						background: Some(Background::Color(bg)),
						border: Border {
							radius: 8.0.into(),
							..Default::default()
						},
						..Default::default()
					}
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
