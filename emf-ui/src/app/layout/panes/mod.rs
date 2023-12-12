use egui::{Id, Key, Sense, Vec2};
use egui_tiles::{Behavior, Tabs, TileId};

use crate::app::app_state::View;

use super::widgets::console::Console;

pub trait PaneView {
	fn ui(&mut self, ui: &mut egui::Ui);

	fn title(&self) -> &str;
}

#[derive(Default)]
struct Config {
	pub visible: bool,
}

#[derive(Default)]
struct TreeCache {
	pub config: Config,
	pub settings: Settings,
	pub console: Console,
}

pub struct TreeParent {
	pub tree: egui_tiles::Tree<Pane>,
	cache: TreeCache,
	pub first_run: bool,
	name: String,
}

impl TreeParent {
	pub fn new(name: &str) -> Self {
		let mut tiles = egui_tiles::Tiles::default();

		let views: Vec<TileId> = vec![
			tiles.insert_pane(Pane::Console),
			tiles.insert_pane(Pane::Settings),
			tiles.insert_pane(Pane::Text("Test".to_string())),
		];

		let root = tiles.insert_container(egui_tiles::Container::Tabs(Tabs::new(views)));

		Self {
			tree: egui_tiles::Tree::new(name.to_owned(), root, tiles),
			cache: TreeCache::default(),
			first_run: true,
			name: name.to_owned(),
		}
	}

	pub fn insert_pane(&mut self, pane: Pane) -> TileId {
		let id = self.tree.tiles.insert_pane(pane);

		self.tree
			.tiles
			.insert_container(egui_tiles::Container::Tabs(Tabs::new(vec![id])))
	}
}

impl View for TreeParent {
	fn show(&mut self, ctx: &egui::Context, visible: &mut bool) {
		if ctx.input(|i| i.key_pressed(Key::F1)) {
			*visible = !*visible;
			dbg!(&visible);
		}

		egui::Window::new(self.name.clone())
			.default_size(Vec2::new(600.0, 400.0))
			.open(visible)
			.show(ctx, |ui| {
				self.ui(ui);
				// ui.label("Meow!");
				ui.allocate_space(ui.available_size());
			});

		// FIXME: This is a hack to make sure the window renders.
		// For some reason, if the window visibility is set to false on the first run,
		// it never renders, even when visibility is toggled to true.
		if self.first_run {
			self.first_run = false;
			*visible = false;
		}
	}

	fn ui(&mut self, ui: &mut egui::Ui) {
		tree_ui(ui, &mut self.tree, &mut self.cache);

		// egui::TopBottomPanel::top(Id::new("tree_parent"))
		// 	.exact_height(ui.available_height() / 3.0)
		// 	.show(ui.ctx(), |ui| {
		// 		tree_ui(ui, &mut self.tree, &mut self.cache);
		// 	})
		// 	.response
	}
}

pub enum Pane {
	Settings,
	Console,
	Text(String),
	Custom(Box<dyn PaneView>),
}

fn tree_ui(ui: &mut egui::Ui, tree: &mut egui_tiles::Tree<Pane>, cache: &mut TreeCache) {
	let mut behavior = OverlayUI {
		config: &mut cache.config,
		settings: &mut cache.settings,
		console: &mut cache.console,
	};
	tree.ui(&mut behavior, ui);
}

struct OverlayUI<'a> {
	config: &'a mut Config,
	settings: &'a mut Settings,
	console: &'a mut Console,
}

impl<'a> Behavior<Pane> for OverlayUI<'a> {
	fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
		match pane {
			Pane::Settings => "Settings".into(),
			Pane::Console => "Console".into(),
			Pane::Text(text) => text.clone().into(),
			Pane::Custom(component) => component.title().into(),
		}
	}

	fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
		egui_tiles::SimplificationOptions {
			all_panes_must_have_tabs: true,
			..Default::default()
		}
	}

	fn pane_ui(
		&mut self,
		ui: &mut egui::Ui,
		_tile_id: egui_tiles::TileId,
		pane: &mut Pane,
	) -> egui_tiles::UiResponse {
		match pane {
			Pane::Settings => self.settings.ui(ui),
			Pane::Console => self.console.ui(ui),
			Pane::Text(text) => {
				ui.text_edit_singleline(text);
			}
			Pane::Custom(component) => component.ui(ui),
		};

		egui_tiles::UiResponse::None
	}
}

#[derive(Default)]
struct Settings {
	checked: bool,
}

impl PaneView for Settings {
	fn title(&self) -> &str {
		"Settings"
	}
	fn ui(&mut self, ui: &mut egui::Ui) {
		ui.checkbox(&mut self.checked, "Checked");
	}
}
