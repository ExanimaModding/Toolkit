use emf_ui::{AppState, Panes, Widgets};

pub fn load_gui() {
	std::thread::spawn(|| {
		// let mut tree_parent = Panes::TreeParent::new("Framework Overlay1");

		// let console = Widgets::console::Console::new("Console1");

		// tree_parent.insert_pane(Panes::Pane::Custom(Box::new(console)));

		// FIXME: For some reason if I initialise, and THEN add a View,
		// the View doesn't get rendered. But the code does run.
		emf_ui::init();

		// unsafe {
		// 	let mut state = AppState.try_lock_state();
		// 	while state.is_none() {
		// 		std::thread::sleep(std::time::Duration::from_millis(100));
		// 		state = AppState.try_lock_state();
		// 	}
		// 	state.unwrap().add_widget(Box::new(tree_parent));
		// }
	});
}
