use crate::internal::gui::Widget;

#[derive(Default)]
pub struct Menu {
	items: Vec<String>,
	pub selected: String,
}

impl Menu {
	pub fn new(items: Vec<String>) -> Self {
		Self {
			items: items.clone(),
			selected: items[0].clone(),
		}
	}
}

impl Widget for Menu {
	fn render(&mut self, ui: &hudhook::imgui::Ui) {
		for (i, item) in self.items.iter().enumerate() {
			ui.button(item);

			if ui.is_item_clicked_with_button(hudhook::imgui::MouseButton::Left) {
				self.selected = item.clone();
			}

			if i < self.items.len() - 1 {
				ui.same_line();
			}
		}
	}
}
