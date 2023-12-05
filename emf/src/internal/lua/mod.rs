use mlua::{chunk, prelude::*, Variadic};

use self::hooks::hook;

pub mod hooks;

pub struct LuaRuntime(Option<Lua>);

impl LuaRuntime {
	pub fn new() -> Self {
		Self(Some(Lua::new()))
	}

	pub fn get(&self) -> &Lua {
		self.0.as_ref().unwrap()
	}
}

/// TODO: Should this be LUA_RUNTIME? But ugly...
#[allow(non_upper_case_globals)]
pub static mut luaRuntime: LuaRuntime = LuaRuntime(None);

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

	let globals = luaRuntime.get().globals();

	// Overwrite the print function to redirect to stdout
	let print = luaRuntime.get().create_function(|_, a: Variadic<String>| {
		println!("{}", a.join(" "));
		Ok(())
	})?;
	globals.set("print", print)?;

	hook()?;

	Ok(())
}
