use detours_sys::{DetourAttach, DetourDetach, DetourTransactionBegin, DetourTransactionCommit};
use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use crate::internal::lua::luaRuntime;

use super::{HookImpl, HookType};

#[derive(Debug)]
#[repr(usize)]
pub enum LuaCodeType {
	Function(usize) = (0),
	Expression,
}

impl Default for LuaCodeType {
	fn default() -> Self {
		Self::Function(0)
	}
}

/// A hook that calls a lua function.
/// Much like MethodHook, this *wraps* the original function.
/// Currently, if it returns true, the original function will be called.
/// If it returns false, the original function will not be called.
#[derive(Default, Debug)]
pub struct LuaHook {
	active: bool,
	original_ptr: usize,
	method_body: Vec<u8>,
	lua_code: String,
	lua_code_type: LuaCodeType,
}

impl LuaHook {
	pub fn new(original_ptr: usize, lua_code: &str, lua_code_type: LuaCodeType) -> Self {
		Self {
			active: false,
			original_ptr,
			method_body: Vec::new(),
			lua_code: lua_code.to_string(),
			lua_code_type,
		}
	}
}

impl HookImpl for LuaHook {
	fn get_type(&self) -> HookType {
		HookType::Lua
	}

	fn get_original_ptr(&self) -> usize {
		self.original_ptr
	}

	fn get_detour_ptr(&self) -> usize {
		self.method_body.as_ptr() as usize
	}

	unsafe fn attach(&mut self) -> bool {
		self.method_body = lua_wrap(&mut self.original_ptr, &(lua_exec as _), self);

		DetourTransactionBegin();
		let result = DetourAttach(
			&mut self.original_ptr as *mut _ as _,
			self.method_body.as_mut_ptr() as _,
		);
		DetourTransactionCommit();
		dbg!(result);

		self.active = result == 0;
		self.active
	}

	unsafe fn detach(&mut self) -> bool {
		DetourTransactionBegin();
		let result = DetourDetach(
			&mut self.original_ptr as *mut _ as _,
			self.method_body.as_mut_ptr() as _,
		);
		DetourTransactionCommit();
		dbg!(result);

		self.active = result == 0;
		self.active
	}
}

#[repr(C)]
#[derive(Debug)]
pub struct LuaArgs {
	hook: *const LuaHook,
	arg1: [usize],
}

#[no_mangle]
pub unsafe extern "cdecl" fn lua_exec(stack: *mut usize) -> bool {
	let hook = *stack.offset(0) as *const LuaHook;

	if hook.is_null() {
		eprint!(
			"[EMF::Internal::Hooking::Lua] Hook at stack pointer {:p} is null in lua_exec",
			stack
		);
		return false;
	}
	let hook = &*hook;

	if let LuaCodeType::Function(arg_count) = hook.lua_code_type {
		let mut code_args = Vec::new();

		for i in 0..arg_count {
			let arg = *stack.offset(i as isize + 1);
			code_args.push(format!("{}", arg));
		}

		let code_args = if code_args.is_empty() {
			"".to_owned()
		} else {
			code_args.join(", ")
		};

		let code = format!("return {}({})", hook.lua_code, code_args);

		return luaRuntime.get().load(code).eval::<bool>().unwrap();
	} else {
		let code = hook.lua_code.to_string();
		return luaRuntime.get().load(code).eval::<bool>().unwrap_or(false);
	}

	// let result: bool = luaRuntime
	// 	.get()
	// 	.load(format!("return {}({})", func, arg2))
	// 	.eval::<bool>()
	// 	.unwrap();

	// result
}

unsafe fn lua_wrap(
	original_fn: *mut usize,
	lua_exec: &*const usize,
	lua_hook: &mut LuaHook,
) -> Vec<u8> {
	let mut ops = dynasmrt::x86::Assembler::new().unwrap();

	dynasm!(ops
		; .arch x86

	  ; push ecx
	  ; push edx
	  ; push eax

		; push lua_hook as *mut _ as _

		// Pass the stack pointer as the first arg, so we can get the remaining args.
		; push esp

	  ; call DWORD [lua_exec as *const _ as _]
		; add esp, 8 // pop esp, pop lua_hook

	  // If function returns true, jump to end
	  ; test eax, eax
	  ; jne >end

	  ; pop eax
	  ; pop edx
	  ; pop ecx

	  ; jmp DWORD [original_fn as *const _ as _]
	  ; retn 8

	  ; end:
	  ; pop eax
	  ; pop edx
	  ; pop ecx
	  ; retn 8
	);

	let buf = ops.finalize().unwrap();

	buf.to_vec()
}
