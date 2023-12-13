use mlua::{chunk, prelude::*};

use crate::internal::lua::mod_loader::get_mods_list;

use self::{
	hooks::{hook, wrap_lua_stdout},
	mod_loader::exec_lua_file,
};

pub mod hooks;
pub mod mod_loader;

pub struct LuaRuntime {
	runtime: Option<Lua>,
}

impl LuaRuntime {
	pub unsafe fn new() -> Self {
		Self {
			runtime: Some(Lua::unsafe_new()),
		}
	}

	pub fn get(&self) -> &Lua {
		self.runtime.as_ref().unwrap()
	}
}

/// TODO: Should this be LUA_RUNTIME? But ugly...
#[allow(non_upper_case_globals)]
pub static mut luaRuntime: LuaRuntime = LuaRuntime { runtime: None };

pub unsafe fn init_lua() -> LuaResult<()> {
	luaRuntime = LuaRuntime::new();

	// Redirect io.write to stdout/stdin
	luaRuntime
		.get()
		.load(chunk! {
			io.output("CONOUT$")
			io.input("CONIN$")
		})
		.exec()?;

	let _globals = luaRuntime.get().globals();

	wrap_lua_stdout()?;

	// Overwrite the print function to redirect to stdout
	// let print = luaRuntime.get().create_function(|_, a: Variadic<String>| {
	// 	println!("{}", a.join(" "));

	// 	Ok(())
	// })?;
	// globals.set("print", print)?;

	hook()?;

	Ok(())
}
