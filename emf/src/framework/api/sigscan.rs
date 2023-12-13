use mlua::Result as LuaResult;

use crate::internal::{lua::luaRuntime, memory::sigscanner::SigScanner};

pub fn init() -> LuaResult<()> {
	let lua = unsafe { luaRuntime.get() };

	let table = lua.create_table()?;

	let scan = lua.create_function(|_, signature: mlua::String| unsafe {
		let signature = signature.to_str().unwrap();
		let result = SigScanner::new(signature).exec();
		if let Some(result) = result.value() {
			Ok(result as usize)
		} else {
			Ok(usize::MAX)
		}
	})?;

	table.set("scan", scan)?;
	table.set("SIG_NOT_FOUND", usize::MAX)?;

	lua.globals().set("SigScan", table)?;

	Ok(())
}
