use mlua::{chunk, prelude::*};

use crate::{
	internal::hooking::{
		hooks::{
			database::HookDB,
			lua::{LuaCodeType, LuaHook},
			Hook, NewHook,
		},
		HookName,
	},
	PROCDMGSTAM_ORIG,
};

use super::luaRuntime;

pub unsafe fn hook() -> LuaResult<()> {
	// let _name = HookName::internal("LuaJit", "prevent_damage");

	// luaRuntime
	// 	.get()
	// 	.load(chunk! {
	// 		first_attacked = 0
	// 		function prevent_damage(actor)
	// 			if first_attacked == 0 or first_attacked == actor then
	// 				first_attacked = actor
	// 				print("Blocking damage for actor:", actor)
	// 				return true
	// 			end

	// 			print("Not blocking damage for actor:", actor)
	// 			return false
	// 		end
	// 	})
	// 	.exec()?;

	// let hook_name = HookName::internal("LuaJIT", "NoDamage");

	// let hook = Hook::new(
	// 	hook_name.to_string(),
	// 	LuaHook::new(
	// 		PROCDMGSTAM_ORIG as usize,
	// 		"prevent_damage",
	// 		LuaCodeType::Function(1),
	// 	),
	// );

	// let hook = HookDB.add_hook(hook);

	// hook.get_hook_mut().attach();

	Ok(())
}
