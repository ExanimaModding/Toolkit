use super::layout::panes;

/// TODO: Widgets should probably be a HashMap<String, Box<dyn View>>
/// However, this would possibly break, as HashMaps cannot be static
/// (unless wrapped in an Option or similar).
pub struct AppState {
	pub widgets: Vec<(Box<dyn View>, bool)>,
}

impl AppState {
	pub fn new_with(widgets: Vec<(Box<dyn View>, bool)>) -> Self {
		Self { widgets }
	}

	pub fn add_widget(&mut self, widget: Box<dyn View>) {
		// TODO: We pass true as the second argument, as we want to show the widget by default.
		// This is because egui_tiles has some weird rendering issue where widgets that
		// are disabled by default, don't get rendered when enabled.
		// The solution is to render it for one frame, then hide it.
		// Hopefully I'll find a fix soon.
		self.widgets.push((widget, true));
		dbg!(self.widgets.len());
	}
}

impl Default for AppState {
	fn default() -> Self {
		Self {
			widgets: vec![(Box::new(panes::TreeParent::new("Framework Overlay")), true)],
			// widgets: Vec::new(),
		}
	}
}

pub trait View {
	fn show(&mut self, ctx: &egui::Context, visible: &mut bool);

	fn ui(&mut self, ctx: &mut egui::Ui);
}
