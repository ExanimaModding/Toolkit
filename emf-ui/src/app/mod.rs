pub mod app_state;
pub mod hook;
pub mod layout;
// use egui::Context;

use std::{sync::mpsc, thread::JoinHandle};

use eframe::NativeOptions;
use egui::{Context, Sense, Ui, Widget};

use crate::AppState;

use self::app_state::{AppState, View};

// TODO: Try this?
// https://github.com/veeenu/hudhook

pub fn render(ctx: &egui::Context, state: &mut AppState) {
	ctx.set_debug_on_hover(true);

	for (widget, visible) in &mut state.widgets {
		widget.show(ctx, visible);
	}
}
