// use std::collections::BTreeMap;

use iced::{
	Border, Element, Fill, Padding, Point, Rectangle, Shrink, Size, Task, Theme,
	advanced::widget as iced_widget,
	border::Radius,
	mouse,
	widget::{
		Space, container, mouse_area, opaque,
		pane_grid::{self, Pane},
		right_center, row, scrollable, stack, text,
	},
};
use iced_drop::droppable;
use tracing::instrument;

use crate::gui::{
	Root,
	buffer::{self, Buffer, instance_history::InstanceHistory},
	widget::{button, close_button, icon, tooltip},
};

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct Id(pub usize);

#[derive(Debug, Clone)]
pub struct Tab {
	pub widget_id: iced_widget::Id,
	pub buffer: Buffer,
	pub loading: bool,
}

impl Tab {
	#[instrument(level = "trace")]
	pub fn new(buffer: Buffer) -> Self {
		Self {
			widget_id: iced_widget::Id::unique(),
			buffer,
			loading: false,
		}
	}
}

// #[derive(Debug, Clone)]
// pub enum Node {
// 	Split { a: Box<Node>, b: Box<Node> },
// 	Tab(Id),
// }

// impl Node {
// 	pub fn peek(&self) {}

// 	pub fn update(&mut self, f: &impl Fn(&mut Node)) {
// 		if let Node::Split { a, b } = self {
// 			a.update(f);
// 			b.update(f);
// 		}

// 		f(self);
// 	}
// }

// #[derive(Debug, Clone)]
// pub struct Internal {
// 	pub focus: Option<usize>,
// 	pub hover: Option<iced_widget::Id>,
// 	pub last_id: usize,
// 	pub layout: Node,
// }

#[derive(Debug, Clone)]
pub struct TabManager {
	// pub internal: Internal,
	// pub tabs: BTreeMap<Id, Tab>,
	pub focus: Option<iced_widget::Id>,
	pub hover: Option<iced_widget::Id>,
	pub over: Option<iced_widget::Id>,
	// PERF: refactor to be a hashmap/btreemap
	pub tabs: Vec<Tab>,
}

#[derive(Debug, Clone)]
pub enum Message {
	Buffer(Pane, buffer::Message),
	ClickedPane(Pane),
	ClickedTab(Pane, iced_widget::Id),
	ClosedPane(Pane),
	ClosedTab(Pane, iced_widget::Id),
	DockTab(Pane),
	DraggedPane(pane_grid::DragEvent),
	DraggedTab(Point, Rectangle),
	DraggedTabCanceled,
	DroppedTab(Point, Rectangle),
	EnteredTabRegion(Pane, iced_widget::Id),
	ExitedTabRegion(Pane, iced_widget::Id),
	NewPane,
	NewTab,
	OverTab(Vec<(iced_widget::Id, Rectangle)>),
	RefreshTab,
	ReorderTabs(Vec<(iced_widget::Id, Rectangle)>),
	Resized(pane_grid::ResizeEvent),
}

impl TabManager {
	#[instrument(level = "trace")]
	pub fn new() -> (Self, Task<buffer::Message>) {
		let (instance_history, task) = InstanceHistory::new();
		let tab = Tab::new(instance_history.into());
		(
			Self {
				focus: Some(tab.widget_id.clone()),
				hover: None,
				over: None,
				tabs: vec![tab],
			},
			task.map(buffer::Message::InstanceHistory),
		)
	}

	// pub fn reorder(&mut self, a: Id, target: Id) {
	// 	self.internal.layout.update(&|node| match node {
	// 		Node::Split { .. } => {}
	// 		Node::Tab(id) => {
	// 			// - if "id" matches "a" first, shift all nodes between node "a" and "target" up
	// 			// a level
	// 			// - if "id" matches "target" first, shift all nodes between node "a" and
	// 			// "target" down a level

	// 			// this is a swap rather than a reorder
	// 			if *id == a {
	// 				*node = Node::Tab(target);
	// 			} else if *id == target {
	// 				*node = Node::Tab(a);
	// 			}
	// 		}
	// 	})
	// }

	#[instrument(level = "trace")]
	pub fn view_header(
		&self,
		_tabs: &pane_grid::State<TabManager>,
		pane: Pane,
		size: Size,
	) -> Element<Message> {
		let btn_size = 28;
		let tab_max_width = 160.;
		let tab_min_width = 60.;
		let tab_width = (size.width - (btn_size * 3) as f32 - 2.) / self.tabs.len() as f32;
		let tab_width = if tab_width < tab_min_width {
			tab_min_width
		} else if tab_width > tab_max_width {
			tab_max_width
		} else {
			tab_width
		};

		let refresh_btn = tooltip(
			button(icon::rotate_cw().size(15).center())
				.width(btn_size)
				.height(btn_size)
				.on_press(Message::RefreshTab),
			text("Refresh"),
			tooltip::Position::Bottom,
		);

		let new_tab_btn = tooltip(
			button(icon::plus().size(12).center())
				.width(btn_size)
				.height(btn_size)
				.on_press(Message::NewTab),
			text("New Tab"),
			tooltip::Position::Bottom,
		);

		let new_pane_btn = tooltip(
			button(icon::plus().size(12).center())
				.width(btn_size)
				.height(btn_size)
				.on_press(Message::NewPane),
			text("New Pane"),
			tooltip::Position::Bottom,
		);

		let tab_elements = row![refresh_btn]
			.extend(self.tabs.iter().map(|tab| {
				let close_tab_btn: Element<_> = if let Some(widget_id) = &self.hover
					&& *widget_id == tab.widget_id
				{
					right_center(
						close_button().on_press(Message::ClosedTab(pane, tab.widget_id.clone())),
					)
					.into()
				} else {
					Space::new(Shrink, Shrink).into()
				};

				droppable(
					mouse_area(tooltip(
						container(
							stack![text(tab.buffer.title()).center(), close_tab_btn]
								.width(Fill)
								.height(Fill),
						)
						.width(tab_width)
						.max_width(tab_max_width as u32)
						.center_y(28)
						.padding([2, 8])
						.style(move |theme: &Theme| {
							let ext_palette = theme.extended_palette();
							let style = container::Style::default()
								.background(ext_palette.background.weak.color)
								.color(ext_palette.background.weak.text)
								.border(Border::default().rounded(Radius::default().top(4)));
							let style = if self.focus.as_ref() == Some(&tab.widget_id) {
								style
									.background(ext_palette.background.strong.color)
									.color(ext_palette.background.strong.text)
							} else {
								style
							};
							if let Some(widget_id) = &self.hover
								&& *widget_id == tab.widget_id
							{
								style
									.background(ext_palette.primary.weak.color)
									.color(ext_palette.primary.weak.text)
							} else {
								style
							}
						}),
						text(if let Buffer::Instance(instance) = &tab.buffer {
							instance.inner().path().display().to_string()
						} else {
							tab.buffer.title()
						}),
						tooltip::Position::Bottom,
					))
					.on_enter(Message::EnteredTabRegion(pane, tab.widget_id.clone()))
					.on_exit(Message::ExitedTabRegion(pane, tab.widget_id.clone()))
					.on_middle_press(Message::ClosedTab(pane, tab.widget_id.clone())),
				)
				.id(tab.widget_id.clone())
				.on_cancel(Message::DraggedTabCanceled)
				.on_click(Message::ClickedTab(pane, tab.widget_id.clone()))
				.on_drag(Message::DraggedTab)
				.on_drop(Message::DroppedTab)
				.drag_hide(true)
				.into()
			}))
			.push(new_tab_btn)
			.push(new_pane_btn)
			.spacing(1);

		scrollable(container(tab_elements).padding(Padding::default().bottom(10)))
			.direction(scrollable::Direction::Horizontal(
				scrollable::Scrollbar::default(),
			))
			.into()
	}

	#[instrument(level = "trace")]
	pub fn view(
		&self,
		_tabs: &pane_grid::State<TabManager>,
		pane: Pane,
		root: &Root,
	) -> Element<Message> {
		if let Some(focus) = &self.focus
			&& let Some(tab) = self.tabs.iter().find(|tab| &tab.widget_id == focus)
		{
			let content = tab
				.buffer
				.view(root)
				.map(move |message| Message::Buffer(pane, message));

			let loading = if tab.loading {
				true
			} else if root.loading
				&& let Buffer::Settings(_) = tab.buffer
			{
				true
			} else {
				false
			};

			if loading {
				stack![
					content,
					opaque(
						mouse_area(Space::new(Fill, Fill)).interaction(mouse::Interaction::Working),
					)
				]
				.into()
			} else {
				content
			}
		} else {
			Space::new(Shrink, Shrink).into()
		}
	}
}
