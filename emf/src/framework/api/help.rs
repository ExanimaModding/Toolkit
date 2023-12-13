use mlua::{chunk, Result as LuaResult};

use crate::internal::lua::luaRuntime;

pub fn load_help_cmd() -> LuaResult<()> {
	let lua = unsafe { luaRuntime.get() };

	lua.load(chunk! {
		function help()
			print(
				"To create a new hook:\n",
				"local hook = hooks.create_hook(module, name, address, code_to_run, args_count)\n",
				"local hook = hooks.get_hook(hook)\n",
				"hook.attach()"
			)
		end
	})
	.exec()?;

	Ok(())
}
