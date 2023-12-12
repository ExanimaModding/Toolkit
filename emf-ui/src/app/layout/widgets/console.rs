use std::os::windows::io::AsHandle;

use egui::{
	Align, Area, CentralPanel, Id, Key, Layout, RichText, ScrollArea, TextEdit, TopBottomPanel,
};

use crate::app::layout::panes::PaneView;

pub struct Console {
	log: Vec<String>,
	stdin: String,
	visible: bool,
	focused: bool,
	title: String,
}

impl Console {
	pub fn new(title: &str) -> Self {
		Self {
			log: vec![],
			stdin: String::new(),
			visible: false,
			focused: false,
			title: title.to_string(),
		}
	}

	pub fn log(&mut self, msg: &str) {
		self.log.push(msg.to_string());
	}

	pub fn write_flush(&mut self) {
		self.log.push(self.stdin.clone());
		self.stdin.clear();
	}

	pub fn toggle_visibility(&mut self) {
		self.visible = !self.visible;
	}
}

impl Default for Console {
	fn default() -> Self {
		Self::new("Console")
	}
}

impl PaneView for Console {
	fn title(&self) -> &str {
		&self.title
	}

	fn ui(&mut self, ui: &mut egui::Ui) {
		self.bottom_panel(ui);
		self.top_panel(ui);
	}
}

impl Console {
	fn top_panel(&mut self, ui: &mut egui::Ui) {
		CentralPanel::default().show_inside(ui, |ui| {
			ScrollArea::both().stick_to_bottom(true).show(ui, |ui| {
				ui.set_width(ui.available_width());
				for i in self.log.iter() {
					ui.label(RichText::new(i).monospace());
				}
			});
		});
	}

	fn bottom_panel(&mut self, ui: &mut egui::Ui) {
		TopBottomPanel::bottom("console_input").show_inside(ui, |ui| {
			let code_editor = TextEdit::singleline(&mut self.stdin)
				.code_editor()
				.desired_width(ui.available_width())
				.clip_text(true);

			let text_box = ui.add(code_editor);

			if text_box.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
				self.write_flush();
				text_box.request_focus();
			}
		});
	}
}
