use std::sync::{Arc, Mutex};

use hudhook::*;

use crate::{internal::gui::Widget, plugins::manager::PluginManager};
use emtk_framework_types::config;

#[derive(Debug)]
pub struct Plugin {
	pub plugin: Arc<Mutex<config::PluginInfo>>,
	pub settings_component: PluginSettings,
	pub current_tab: PluginTab,
}

#[derive(Default, Debug)]
pub enum PluginTab {
	#[default]
	Info,
	Settings,
}

impl From<config::PluginInfo> for Plugin {
	fn from(plugin: config::PluginInfo) -> Self {
		let plugin = Arc::new(Mutex::new(plugin));

		let settings_component = PluginSettings::new(plugin.clone());

		Self {
			plugin,
			settings_component,
			current_tab: PluginTab::Info,
		}
	}
}

impl Widget for Plugin {
	fn initialize(&mut self, _ctx: &mut imgui::Context, _render_context: &mut dyn RenderContext) {
		self.settings_component.initialize(_ctx, _render_context);
	}

	fn render(&mut self, ui: &imgui::Ui) {
		{
			let plugin = self.plugin.lock().unwrap();

			let status = if plugin.config.plugin.enabled {
				"Enabled"
			} else {
				"Disabled"
			};

			ui.text_wrapped(format!("{} ({})", plugin.config.plugin.name, status));
			ui.button("Info");
			if ui.is_item_clicked() {
				self.current_tab = PluginTab::Info;
			}

			ui.same_line();
			ui.button("Settings");
			if ui.is_item_clicked() {
				self.current_tab = PluginTab::Settings;
			}
			ui.separator();
		}

		match self.current_tab {
			PluginTab::Info => {
				let plugin = self.plugin.lock().unwrap();

				ui.text_wrapped(format!("Version: {}", plugin.config.plugin.version));

				ui.text_wrapped(format!("Author: {}", plugin.config.plugin.author.name));

				if let Some(contact) = &plugin.config.plugin.author.contact {
					if contact.starts_with("https://") || contact.starts_with("http://") {
						if ui.button("Contact") {
							open::that(contact).unwrap();
						}
					} else {
						ui.text_wrapped(format!("Contact: {}", contact));
					}
				}

				if ui.button("View Source") {
					open::that(&plugin.config.plugin.url).unwrap();
				}

				ui.separator();

				if let Some(description) = &plugin.config.plugin.description {
					ui.text_wrapped(description);
				}
			}
			PluginTab::Settings => {
				self.settings_component.render(ui);
			}
		}
	}
}

#[derive(Debug)]
pub struct PluginSettings {
	pub plugin: Arc<Mutex<config::PluginInfo>>,
	pub inputs: Vec<SettingInput>,
	pub settings_changed: bool,
	pub enabled: bool,
}

impl PluginSettings {
	pub fn new(plugin: Arc<Mutex<config::PluginInfo>>) -> Self {
		let enabled = plugin.clone().lock().unwrap().config.plugin.enabled;

		Self {
			plugin,
			inputs: vec![],
			settings_changed: false,
			enabled,
		}
	}
}

impl Widget for PluginSettings {
	fn initialize(&mut self, _ctx: &mut imgui::Context, _render_context: &mut dyn RenderContext) {
		let plugin = self.plugin.lock().unwrap();
		for setting in plugin.config.settings.iter() {
			self.inputs.push(SettingInput {
				value: setting.value.clone().unwrap(),
				name: setting.name.clone(),
			});
		}
	}

	fn render(&mut self, ui: &imgui::Ui) {
		let mut plugin = self.plugin.lock().unwrap();

		ui.checkbox("Enable Plugin", &mut self.enabled);

		{
			let mut is_any_changed = false;
			if self.enabled != plugin.config.plugin.enabled {
				is_any_changed = true;
			}
			for (i, _) in plugin.config.settings.iter().enumerate() {
				if self.inputs[i].value != plugin.config.settings[i].value.clone().unwrap() {
					is_any_changed = true;
				}
			}
			self.settings_changed = is_any_changed;
		}

		if plugin.config.settings.is_empty() {
			ui.text("No settings available for this plugin.");
		} else if let Some(_table) = ui.begin_table_with_flags(
			format!("Settings##Settings:{}", &plugin.config.plugin.name),
			2,
			imgui::TableFlags::BORDERS | imgui::TableFlags::SIZING_STRETCH_PROP,
		) {
			ui.table_setup_column("Setting");
			ui.table_setup_column("Value");
			ui.table_headers_row();

			for setting in self.inputs.iter_mut() {
				ui.table_next_column();
				ui.text(setting.name.as_str());

				ui.table_next_column();
				setting.render(ui);
			}
		}

		ui.enabled(self.settings_changed, || {
			ui.button("Save");
			if ui.is_item_clicked() {
				plugin.config.plugin.enabled = self.enabled;

				for (i, setting) in plugin.config.settings.iter_mut().enumerate() {
					setting.value = Some(self.inputs[i].value.clone());
				}

				PluginManager::set_info_for(&plugin.config.plugin.id, (*plugin).clone());
			}
		});

		ui.button("Reset to Defaults");

		let popup_id = format!(
			"Reset to Defaults?##ResetToDefaults:{}",
			plugin.config.plugin.id
		);

		ui.popup(&popup_id, || {
			ui.text("Are you sure you want to reset all settings to their default values?");
			ui.button("Reset Settings");
			if ui.is_item_clicked() {
				plugin.config.plugin.enabled = self.enabled;
				for (i, setting) in plugin.config.settings.iter_mut().enumerate() {
					setting.value = Some(setting.default.clone());
					self.inputs[i].value = setting.default.clone();
				}
				PluginManager::set_info_for(&plugin.config.plugin.id, (*plugin).clone());
				ui.close_current_popup();
			}
			ui.same_line();
			ui.button("Don't Reset Settings");
			if ui.is_item_clicked() {
				ui.close_current_popup();
			}
		});

		if ui.is_item_clicked() {
			ui.open_popup(&popup_id);
		}
	}
}

#[derive(Debug)]
pub struct SettingInput {
	pub value: config::PluginConfigSettingValue,
	pub name: String,
}

impl Widget for SettingInput {
	fn render(&mut self, ui: &imgui::Ui) {
		let id = format!("##{}", self.name);
		match &mut self.value {
			config::PluginConfigSettingValue::Boolean(value) => {
				ui.checkbox(id, value);
			}
			config::PluginConfigSettingValue::Integer(value) => {
				ui.set_next_item_width(ui.content_region_avail()[0]);
				ui.input_scalar(id, value).build();
			}
			config::PluginConfigSettingValue::Float(value) => {
				ui.set_next_item_width(ui.content_region_avail()[0]);
				ui.input_scalar(id, value).build();
			}
			config::PluginConfigSettingValue::String(value) => {
				ui.set_next_item_width(ui.content_region_avail()[0]);
				ui.input_text(id, value).build();
			}
		}
	}
}
