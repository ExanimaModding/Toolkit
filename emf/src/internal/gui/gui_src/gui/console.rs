use crate::internal::{
	gui::gui_src::{MyRenderLoop, RenderFn},
	lua::luaRuntime,
};
use hudhook::imgui;
use std::fmt::{Display, Formatter};

use super::WindowState;

pub struct Console {
	window_state: WindowState,
	pub visible: bool,
	pub lua_print_fn: Option<mlua::Function<'static>>,
	sticky_scroll: bool,
	last_length: usize,
	current_input: String,
	logs: Vec<LogEntry>,
}
unsafe impl Send for Console {}
unsafe impl Sync for Console {}

impl Default for Console {
	fn default() -> Self {
		Self {
			window_state: WindowState::new(true, [600., 300.], [0., 0.]),
			visible: true,
			lua_print_fn: None,
			sticky_scroll: true,
			last_length: 0,
			current_input: String::new(),
			logs: Vec::new(),
		}
	}
}

impl RenderFn for Console {
	fn render(&mut self, renderer: &mut MyRenderLoop, ui: &mut hudhook::imgui::Ui) {
		{
			// Since the window is at the top-right, and relies on display_size, we need to set the initial position manually.
			self.window_state.before_initialize(ui, |ui, window_state| {
				let display_size = ui.io().display_size;
				window_state.position = [display_size[0] - 600. - 32., 32.];
			});

			// If the backtick key is pressed, toggle console visibility.
			// 223 is for ISO layout | 192 is for ANSI layout
			if ui.is_key_index_pressed(223) || ui.is_key_index_pressed(192) {
				self.window_state.toggle_visible();
			}

			// Return early if the console is not visible.
			if !self.window_state.visible {
				return;
			}
		}

		// Gets the current state - this fixes window positions on resize/fullscreen toggle.
		let window_state: WindowState = self.window_state.get_state(ui);
		ui.window("Console")
			.scroll_bar(false)
			.scrollable(false)
			.position(window_state.position, imgui::Condition::FirstUseEver)
			.size(window_state.size, imgui::Condition::FirstUseEver)
			.build(|| {
				self.window_state.store_position(ui);
				self.window_state.store_size(ui);

				self.render_logs(renderer, &ui);
				ui.separator();
				self.render_input(renderer, &ui);
			});
	}
}

impl Console {
	pub fn add_log(&mut self, input: String, direction: LogEntryDirection) {
		self.logs.push(LogEntry {
			text: input.clone(),
			kind: LogEntryKind::Info,
			direction,
		});
	}
	pub fn exec_lua(&mut self, input: String, direction: LogEntryDirection) {
		self.logs.push(LogEntry {
			text: input.clone(),
			kind: LogEntryKind::Info,
			direction,
		});

		let result = unsafe { luaRuntime.get() }.load(&input).exec();

		if result.is_err() {
			self.logs.push(LogEntry {
				text: format!("Error: {}", result.err().unwrap()),
				kind: LogEntryKind::Error,
				direction: LogEntryDirection::Output,
			});
		}
	}

	pub fn render_logs(&mut self, _renderer: &mut MyRenderLoop, ui: &&mut hudhook::imgui::Ui) {
		ui.child_window("Message Log")
			.scroll_bar(true)
			.scrollable(true)
			.size([0., -25.])
			.build(|| {
				for entry in self.logs.iter() {
					ui.text_wrapped(&entry.to_string());
				}

				if ui.is_window_appearing() {
					ui.set_scroll_here_y();
				}

				self.sticky_scroll = ui.scroll_y() == ui.scroll_max_y();

				if self.last_length < self.logs.len() {
					self.last_length = self.logs.len();
					if self.sticky_scroll {
						ui.set_scroll_here_y();
					}
				}
			});
	}

	pub fn render_input(&mut self, renderer: &mut MyRenderLoop, ui: &&mut hudhook::imgui::Ui) {
		ui.set_next_item_width(ui.window_size()[0]);
		ui.input_text("##text_input", &mut self.current_input)
			.build();
		ui.set_item_default_focus();

		renderer.should_block_messages = ui.is_item_active();

		if ui.is_item_focused() && ui.is_key_pressed(imgui::Key::Enter) {
			if self.current_input.is_empty() {
				ui.set_keyboard_focus_here_with_offset(imgui::FocusedWidget::Previous);
				return;
			}
			self.exec_lua(self.current_input.to_owned(), LogEntryDirection::Input);
			self.current_input.clear();
			ui.set_keyboard_focus_here_with_offset(imgui::FocusedWidget::Previous);
		}
	}
}

pub enum LogEntryDirection {
	Input,
	Output,
}

enum LogEntryKind {
	Info,
	#[allow(unused)]
	Warning,
	Error,
}

struct LogEntry {
	text: String,
	direction: LogEntryDirection,
	#[allow(unused)]
	kind: LogEntryKind,
}

impl Display for LogEntry {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.direction {
			LogEntryDirection::Input => write!(f, "<< {}", self.text),
			LogEntryDirection::Output => write!(f, ">> {}", self.text),
		}
	}
}
