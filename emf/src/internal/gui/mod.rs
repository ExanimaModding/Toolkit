mod gui_src;
use gui_src::gui::console::Console;
use mlua::chunk;

use crate::internal::gui::gui_src::gui::console::LogEntryDirection;

use super::lua::luaRuntime;

pub unsafe fn inject_gui() {
	gui_src::inject();

	let gui = gui_src::EMF_GUI.get_mut().unwrap();

	let console: Box<Console> = Box::<Console>::default();

	let runtime = unsafe { luaRuntime.get() };

	let index = gui.add_component(console);
	runtime
		.globals()
		.set(
			"__redirected_outputs__imgui_console__",
			runtime
				.create_function(move |_, a: mlua::Variadic<String>| {
					let gui = gui_src::EMF_GUI.get_mut().unwrap();
					let console = gui.components.get_mut(index).unwrap().as_mut();
					let console = console as *mut _ as *mut Console;
					let console = unsafe { &mut *console };
					console.add_log(a.join(" "), LogEntryDirection::Output);
					Ok(())
				})
				.unwrap(),
		)
		.unwrap();

	runtime
		.load(chunk! {
			table.insert(__redirected_outputs__, __redirected_outputs__imgui_console__)
		})
		.exec()
		.unwrap();
}
