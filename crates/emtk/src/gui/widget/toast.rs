use std::{
	collections::HashMap,
	fmt,
	time::{Duration, Instant},
};

use iced::{
	advanced::{
		layout::{self, Layout},
		overlay, renderer,
		widget::{self, Operation, Tree},
		Clipboard, Shell, Widget,
	},
	event::{self, Event},
	mouse,
	widget::{button, container, svg, text, Column, Row},
	window, Alignment, Border, Center, Color, Element, Fill, Length, Point, Rectangle, Renderer,
	Shadow, Size, Theme, Vector,
};
use strum::EnumIter;

use crate::gui::theme;

pub const DEFAULT_TIMEOUT: u64 = 10;

#[derive(Debug, Hash, PartialEq, Eq, EnumIter)]
pub enum Icon {
	Danger,
	Info,
	Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
	#[default]
	Primary,
	Secondary,
	Success,
	Danger,
}

impl Status {
	pub const ALL: &'static [Self] = &[Self::Primary, Self::Secondary, Self::Success, Self::Danger];
}

impl fmt::Display for Status {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Status::Primary => "Primary",
			Status::Secondary => "Secondary",
			Status::Success => "Success",
			Status::Danger => "Danger",
		}
		.fmt(f)
	}
}

#[derive(Debug, Clone, Default)]
pub struct Toast {
	pub title: String,
	pub body: String,
	pub status: Status,
}

pub struct Manager<'a, Message> {
	content: Element<'a, Message>,
	toasts: Vec<Element<'a, Message>>,
	timeout_secs: u64,
	on_close: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message> Manager<'a, Message>
where
	Message: 'a + Clone,
{
	pub fn new(
		content: impl Into<Element<'a, Message>>,
		toasts: &'a [Toast],
		on_close: impl Fn(usize) -> Message + 'a,
		icons: Option<HashMap<Icon, svg::Handle>>,
	) -> Self {
		let toasts = toasts
			.iter()
			.enumerate()
			.map(|(index, toast)| {
				container(
					button(
						Column::new()
							.push(
								Row::new()
									.push_maybe(match &icons {
										Some(icons) => {
											let icon = match toast.status {
												Status::Primary => icons.get(&Icon::Info),
												Status::Secondary => icons.get(&Icon::Info),
												Status::Success => icons.get(&Icon::Success),
												Status::Danger => icons.get(&Icon::Danger),
											};
											match icon {
												Some(handle) => Some(
													svg(handle.clone())
														.width(Length::Shrink)
														.style(match toast.status {
															Status::Primary => theme::svg_primary,
															Status::Secondary => {
																theme::svg_secondary
															}
															Status::Success => theme::svg_success,
															Status::Danger => theme::svg_danger,
														}),
												),
												None => None,
											}
										}
										None => None,
									})
									.push(text(toast.title.as_str()))
									.align_y(Center)
									.spacing(6),
							)
							.push_maybe(if toast.body.is_empty() {
								None
							} else {
								Some(text(toast.body.as_str()))
							})
							.width(Fill)
							.padding(6),
					)
					.on_press((on_close)(index))
					.padding(0)
					.style(|theme: &Theme, status| {
						let palette = theme.extended_palette();

						let mut style = theme::transparent_button(theme, status)
							.with_background(palette.background.base.color);
						style.border = Border::default()
							.color(palette.background.weak.color)
							.width(1.);
						style.shadow = Shadow {
							color: Color::BLACK,
							offset: Vector::new(2., 2.),
							blur_radius: 8.,
						};
						style
					}),
				)
				.max_width(300)
				.into()
			})
			.collect();

		Self {
			content: content.into(),
			toasts,
			timeout_secs: DEFAULT_TIMEOUT,
			on_close: Box::new(on_close),
		}
	}

	pub fn timeout(self, seconds: u64) -> Self {
		Self {
			timeout_secs: seconds,
			..self
		}
	}
}

impl<'a, Message> Widget<Message, Theme, Renderer> for Manager<'a, Message> {
	fn size(&self) -> Size<Length> {
		self.content.as_widget().size()
	}

	fn layout(
		&self,
		tree: &mut Tree,
		renderer: &Renderer,
		limits: &layout::Limits,
	) -> layout::Node {
		self.content
			.as_widget()
			.layout(&mut tree.children[0], renderer, limits)
	}

	fn tag(&self) -> widget::tree::Tag {
		struct Marker;
		widget::tree::Tag::of::<Marker>()
	}

	fn state(&self) -> widget::tree::State {
		widget::tree::State::new(Vec::<Option<Instant>>::new())
	}

	fn children(&self) -> Vec<Tree> {
		std::iter::once(Tree::new(&self.content))
			.chain(self.toasts.iter().map(Tree::new))
			.collect()
	}

	fn diff(&self, tree: &mut Tree) {
		let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

		// Invalidating removed instants to None allows us to remove
		// them here so that diffing for removed / new toast instants
		// is accurate
		instants.retain(Option::is_some);

		match (instants.len(), self.toasts.len()) {
			(old, new) if old > new => {
				instants.truncate(new);
			}
			(old, new) if old < new => {
				instants.extend(std::iter::repeat(Some(Instant::now())).take(new - old));
			}
			_ => {}
		}

		tree.diff_children(
			&std::iter::once(&self.content)
				.chain(self.toasts.iter())
				.collect::<Vec<_>>(),
		);
	}

	fn operate(
		&self,
		state: &mut Tree,
		layout: Layout<'_>,
		renderer: &Renderer,
		operation: &mut dyn Operation,
	) {
		operation.container(None, layout.bounds(), &mut |operation| {
			self.content
				.as_widget()
				.operate(&mut state.children[0], layout, renderer, operation);
		});
	}

	fn on_event(
		&mut self,
		state: &mut Tree,
		event: Event,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		renderer: &Renderer,
		clipboard: &mut dyn Clipboard,
		shell: &mut Shell<'_, Message>,
		viewport: &Rectangle,
	) -> event::Status {
		self.content.as_widget_mut().on_event(
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
		state: &Tree,
		renderer: &mut Renderer,
		theme: &Theme,
		style: &renderer::Style,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		viewport: &Rectangle,
	) {
		self.content.as_widget().draw(
			&state.children[0],
			renderer,
			theme,
			style,
			layout,
			cursor,
			viewport,
		);
	}

	fn mouse_interaction(
		&self,
		state: &Tree,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		viewport: &Rectangle,
		renderer: &Renderer,
	) -> mouse::Interaction {
		self.content.as_widget().mouse_interaction(
			&state.children[0],
			layout,
			cursor,
			viewport,
			renderer,
		)
	}

	fn overlay<'b>(
		&'b mut self,
		state: &'b mut Tree,
		layout: Layout<'_>,
		renderer: &Renderer,
		translation: Vector,
	) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
		let instants = state.state.downcast_mut::<Vec<Option<Instant>>>();

		let (content_state, toasts_state) = state.children.split_at_mut(1);

		let content = self.content.as_widget_mut().overlay(
			&mut content_state[0],
			layout,
			renderer,
			translation,
		);

		let toasts = (!self.toasts.is_empty()).then(|| {
			overlay::Element::new(Box::new(Overlay {
				position: layout.bounds().position() + translation,
				toasts: &mut self.toasts,
				state: toasts_state,
				instants,
				on_close: &self.on_close,
				timeout_secs: self.timeout_secs,
			}))
		});
		let overlays = content.into_iter().chain(toasts).collect::<Vec<_>>();

		(!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
	}
}

struct Overlay<'a, 'b, Message> {
	position: Point,
	toasts: &'b mut [Element<'a, Message>],
	state: &'b mut [Tree],
	instants: &'b mut [Option<Instant>],
	on_close: &'b dyn Fn(usize) -> Message,
	timeout_secs: u64,
}

impl<'a, 'b, Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'a, 'b, Message> {
	fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
		let limits = layout::Limits::new(Size::ZERO, bounds);

		layout::flex::resolve(
			layout::flex::Axis::Vertical,
			renderer,
			&limits,
			Fill,
			Fill,
			10.into(),
			10.0,
			Alignment::End,
			self.toasts,
			self.state,
		)
		.translate(Vector::new(self.position.x, self.position.y))
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
		if let Event::Window(window::Event::RedrawRequested(now)) = &event {
			let mut next_redraw: Option<window::RedrawRequest> = None;

			self.instants
				.iter_mut()
				.enumerate()
				.for_each(|(index, maybe_instant)| {
					if let Some(instant) = maybe_instant.as_mut() {
						let remaining = Duration::from_secs(self.timeout_secs)
							.saturating_sub(instant.elapsed());

						if remaining == Duration::ZERO {
							maybe_instant.take();
							shell.publish((self.on_close)(index));
							next_redraw = Some(window::RedrawRequest::NextFrame);
						} else {
							let redraw_at = window::RedrawRequest::At(*now + remaining);
							next_redraw = next_redraw
								.map(|redraw| redraw.min(redraw_at))
								.or(Some(redraw_at));
						}
					}
				});

			if let Some(redraw) = next_redraw {
				shell.request_redraw(redraw);
			}
		}

		let viewport = layout.bounds();

		self.toasts
			.iter_mut()
			.zip(self.state.iter_mut())
			.zip(layout.children())
			.zip(self.instants.iter_mut())
			.map(|(((child, state), layout), instant)| {
				let mut local_messages = vec![];
				let mut local_shell = Shell::new(&mut local_messages);

				let status = child.as_widget_mut().on_event(
					state,
					event.clone(),
					layout,
					cursor,
					renderer,
					clipboard,
					&mut local_shell,
					&viewport,
				);

				if !local_shell.is_empty() {
					instant.take();
				}

				shell.merge(local_shell, std::convert::identity);

				status
			})
			.fold(event::Status::Ignored, event::Status::merge)
	}

	fn draw(
		&self,
		renderer: &mut Renderer,
		theme: &Theme,
		style: &renderer::Style,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
	) {
		let viewport = layout.bounds();

		for ((child, state), layout) in self
			.toasts
			.iter()
			.zip(self.state.iter())
			.zip(layout.children())
		{
			child
				.as_widget()
				.draw(state, renderer, theme, style, layout, cursor, &viewport);
		}
	}

	fn operate(
		&mut self,
		layout: Layout<'_>,
		renderer: &Renderer,
		operation: &mut dyn widget::Operation,
	) {
		operation.container(None, layout.bounds(), &mut |operation| {
			self.toasts
				.iter()
				.zip(self.state.iter_mut())
				.zip(layout.children())
				.for_each(|((child, state), layout)| {
					child
						.as_widget()
						.operate(state, layout, renderer, operation);
				});
		});
	}

	fn mouse_interaction(
		&self,
		layout: Layout<'_>,
		cursor: mouse::Cursor,
		viewport: &Rectangle,
		renderer: &Renderer,
	) -> mouse::Interaction {
		self.toasts
			.iter()
			.zip(self.state.iter())
			.zip(layout.children())
			.map(|((child, state), layout)| {
				child
					.as_widget()
					.mouse_interaction(state, layout, cursor, viewport, renderer)
			})
			.max()
			.unwrap_or_default()
	}

	fn is_over(&self, layout: Layout<'_>, _renderer: &Renderer, cursor_position: Point) -> bool {
		layout
			.children()
			.any(|layout| layout.bounds().contains(cursor_position))
	}
}

impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
where
	Message: 'a,
{
	fn from(manager: Manager<'a, Message>) -> Self {
		Element::new(manager)
	}
}

fn styled(pair: iced::theme::palette::Pair) -> container::Style {
	container::Style {
		text_color: pair.text.into(),
		background: Some(pair.color.into()),
		..Default::default()
	}
}

fn primary(theme: &Theme) -> container::Style {
	let palette = theme.extended_palette();

	styled(palette.primary.strong)
}

fn secondary(theme: &Theme) -> container::Style {
	let palette = theme.extended_palette();

	styled(palette.secondary.strong)
}

fn success(theme: &Theme) -> container::Style {
	let palette = theme.extended_palette();

	styled(palette.success.strong)
}

fn danger(theme: &Theme) -> container::Style {
	let palette = theme.extended_palette();

	styled(palette.danger.strong)
}
