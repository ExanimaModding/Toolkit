use mlua::prelude::*;

use crate::internal::{
	hooking::{
		hooks::{
			database::HookDB,
			lua::{LuaCodeType, LuaHook},
			Hook, NewHook,
		},
		HookName,
	},
	lua::{luaRuntime, mod_loader},
};

pub unsafe fn init() -> LuaResult<()> {
	let runtime = luaRuntime.get();

	let hooks_table = runtime.create_table().unwrap();

	let get_hook = runtime.create_function(|_, name: String| {
		let table = luaRuntime.get().create_table()?;

		let name_attach = name.to_owned();
		let attach = luaRuntime
			.get()
			.create_function(move |_, ()| Ok(HookDB.attach_hook(&name_attach)))?;

		table.set("attach", attach)?;

		let name_detach = name.to_owned();
		let detach = luaRuntime
			.get()
			.create_function(move |_, ()| Ok(HookDB.detach_hook(&name_detach)))?;

		table.set("detach", detach)?;

		table.set("name", name)?;

		Ok(table)
	})?;

	hooks_table.set("get_hook", get_hook)?;

	let create_hook = runtime.create_function(
		|_,
		 (module, name, address, code_to_run, args_count): (
			String,
			String,
			usize,
			String,
			usize,
		)| {
			let hook_name = HookName::user(
				&module,
				&format!("{}::{}::{}::{}", name, address, code_to_run, args_count),
			);

			let hook = Hook::new(
				hook_name.to_string(),
				LuaHook::new(address, &code_to_run, LuaCodeType::Function(args_count)),
			);

			HookDB.add_hook(hook);

			Ok(hook_name.to_string())
		},
	)?;

	hooks_table.set("create_hook", create_hook)?;

	runtime.globals().set("Hooks", hooks_table)?;

	let mods = mod_loader::get_mods_list()?;

	for path in mods {
		mod_loader::exec_lua_file(&path)?;
	}

	Ok(())
}
