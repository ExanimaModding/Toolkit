use std::collections::HashMap;

use crate::internal::gui::Widget;
use hudhook::*;

use super::Plugin;

pub struct Plugins {
	plugins: HashMap<String, Plugin>,
	selected: String,
}

impl From<Vec<Plugin>> for Plugins {
	fn from(plugins: Vec<Plugin>) -> Self {
		let selected = plugins[0].plugin.lock().unwrap().config.plugin.id.clone();

		let plugins = plugins
			.into_iter()
			.map(|plugin| {
				let id = plugin.plugin.lock().unwrap().config.plugin.id.clone();
				(id, plugin)
			})
			.collect();

		Self { plugins, selected }
	}
}

impl Widget for Plugins {
	fn initialize(&mut self, _ctx: &mut imgui::Context, _render_context: &mut dyn RenderContext) {
		for (_id, plugin) in self.plugins.iter_mut() {
			plugin.initialize(_ctx, _render_context);
		}
	}

	fn render(&mut self, ui: &imgui::Ui) {
		ui.set_next_item_width(200.);
		ui.child_window("Plugins")
			.size([200., 0.])
			.border(true)
			.build(|| {
				for (_id, plugin) in self.plugins.iter() {
					let plugin = plugin.plugin.lock().unwrap();

					let style = if plugin.config.plugin.enabled {
						ui.push_style_color(imgui::StyleColor::Text, [0., 255., 0., 255.])
					} else {
						ui.push_style_color(imgui::StyleColor::Text, [255., 0., 0., 255.])
					};

					if ui
						.selectable_config(plugin.config.plugin.name.clone())
						.selected(self.selected == plugin.config.plugin.id)
						.build()
					{
						self.selected = plugin.config.plugin.id.clone();
					}

					style.pop();
					if ui.is_item_hovered() {
						ui.tooltip_text(&plugin.config.plugin.name)
					}
				}
			});

		ui.same_line();

		let plugin = self.plugins.get_mut(&self.selected);

		if let Some(plugin) = plugin {
			ui.child_window("Plugin Settings").border(true).build(|| {
				plugin.render(ui);
			});
		} else {
			ui.text("No plugin selected");
		}
		// }
	}
}
