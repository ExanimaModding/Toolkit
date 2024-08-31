use crate::gui::GetLatestReleaseState;
use iced::{
	widget::{container, markdown, scrollable, text, Column},
	Element, Size, Task,
};

pub enum Action {
	LinkClicked(String),
	None,
}

#[derive(Debug, Clone)]
pub struct Changelog {
	pub content: Vec<markdown::Item>,
	pub latest_release: GetLatestReleaseState,
	pub size: Option<Size>,
}

#[derive(Debug, Clone)]
pub enum Message {
	LinkClicked(String),
	SizeChanged(Size),
}

impl Changelog {
	pub fn new(
		content: Vec<markdown::Item>,
		latest_release: GetLatestReleaseState,
		size: Option<Size>,
	) -> Self {
		Self {
			content,
			latest_release,
			size,
		}
	}

	pub fn update(&mut self, message: Message) -> (Task<Message>, Action) {
		match message {
			Message::LinkClicked(url) => (Task::none(), Action::LinkClicked(url)),
			Message::SizeChanged(size) => {
				self.size = Some(size);
				(Task::none(), Action::None)
			}
		}
	}

	pub fn view(&self) -> Element<Message> {
		let loading_con = container(text("Checking for updates..."))
			.padding(12)
			.style(|_theme| {
				let palette = iced::theme::Palette::CATPPUCCIN_FRAPPE;
				container::Style {
					text_color: Some(palette.text),
					background: Some(iced::Background::Color(palette.background)),
					border: iced::Border {
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
				let palette = iced::theme::Palette::CATPPUCCIN_FRAPPE;
				let con = container(
					Column::new()
						.push(scrollable(
							markdown(
								&self.content,
								markdown::Settings::default(),
								markdown::Style {
									inline_code_highlight: markdown::Highlight {
										background: iced::Background::Color(palette.background),
										border: iced::Border::default(),
									},
									inline_code_padding: iced::Padding::default(),
									inline_code_color: palette.text,
									link_color: palette.primary,
								},
							)
							.map(|url| Message::LinkClicked(url.to_string())),
						))
						.spacing(10.),
				)
				.padding(12)
				.style(|_theme| {
					let palette = iced::theme::Palette::CATPPUCCIN_FRAPPE;
					container::Style {
						text_color: Some(palette.text),
						background: Some(iced::Background::Color(palette.background)),
						border: iced::Border {
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
