use mlua::{chunk, prelude::*};

use super::luaRuntime;

pub unsafe fn hook() -> LuaResult<()> {
	Ok(())
}

/// Redirects io.write and print to stdout
///
/// To add a new output, push a function to the `__redirected_outputs__` table.
/// e.g.
/// ```lua
/// table.insert(__redirected_outputs__, function (...)
///    -- Do something with the output
/// end)
/// ```
pub unsafe fn wrap_lua_stdout() -> LuaResult<()> {
	luaRuntime
		.get()
		.load(chunk! {
			__redirected_outputs__ = {}
			function io.write(...)
				for _, output in ipairs(__redirected_outputs__) do
					output(...)
				end
			end

			function print(...)
				for _, output in ipairs(__redirected_outputs__) do
					output(table.concat({...}, " "))
				end
			end
		})
		.exec()
}
