use hudhook::imgui;

pub mod console;

#[derive(Default, Debug, Clone, Copy)]
pub struct WindowState {
	pub initialized: bool,
	pub visible: bool,
	pub size: [f32; 2],
	pub position: [f32; 2],
}

impl WindowState {
	pub fn new(visible: bool, size: [f32; 2], position: [f32; 2]) -> Self {
		Self {
			initialized: false,
			visible,
			size,
			position,
		}
	}

	pub fn before_initialize<F>(&mut self, ui: &mut imgui::Ui, func: F)
	where
		F: FnOnce(&mut imgui::Ui, &mut Self),
	{
		if !self.initialized {
			func(ui, self);
		}
	}

	pub fn toggle_visible(&mut self) {
		self.visible = !self.visible;
	}

	#[allow(unused)]
	pub fn store_visible(&mut self, visible: bool) {
		self.visible = visible;
	}

	pub fn store_position(&mut self, ui: &imgui::Ui) {
		self.position = ui.window_pos();
	}

	pub fn store_size(&mut self, ui: &imgui::Ui) {
		self.size = ui.window_size();
	}

	pub fn get_state(&mut self, ui: &mut imgui::Ui) -> Self {
		if ui.is_window_appearing() {
			self.initialized = true;
		}

		*self
	}
}
