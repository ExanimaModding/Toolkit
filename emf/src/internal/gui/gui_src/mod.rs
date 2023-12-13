pub mod gui;
use std::cell::OnceCell;

use hudhook::{
	hooks::opengl3::ImguiOpenGl3Hooks,
	imgui::{self, Ui},
	Hudhook, ImguiRenderLoop,
};

pub trait RenderFn {
	fn render(&mut self, renderer: &mut MyRenderLoop, ui: &mut Ui);
}

#[derive(Default)]
pub struct EmfGui {
	pub components: Vec<Box<dyn RenderFn>>,
}

impl EmfGui {
	pub fn add_component(&mut self, component: Box<dyn RenderFn>) -> usize {
		self.components.push(component);
		self.components.len() - 1
	}
}

pub static mut EMF_GUI: OnceCell<EmfGui> = OnceCell::new();

#[derive(Default)]
pub struct MyRenderLoop {
	should_block_messages: bool,
}

impl ImguiRenderLoop for MyRenderLoop {
	fn initialize(&mut self, ctx: &mut imgui::Context) {
		ctx.io_mut().config_windows_move_from_title_bar_only = true;
	}

	fn should_block_messages(&self, io: &imgui::Io) -> bool {
		io.want_capture_mouse || io.want_capture_keyboard || io.want_text_input
	}

	fn render(&mut self, ui: &mut imgui::Ui) {
		unsafe {
			for component in EMF_GUI.get_mut().unwrap().components.iter_mut() {
				component.render(self, ui);
			}
		}
	}
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn inject() {
	if EMF_GUI.set(EmfGui::default()).is_err() {
		eprintln!("Failed to set EMF_GUI");
	}

	::std::thread::spawn(move || {
		let result = Hudhook::builder()
			.with(MyRenderLoop::default().into_hook::<ImguiOpenGl3Hooks>())
			.build()
			.apply();

		if let Err(e) = result {
			eprintln!("Failed to apply HUD hook: {:?}", e);
		}
	});
}
