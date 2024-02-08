use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

// TODO: Adapt this to the new WASM plugin system.
#[allow(unused)]
struct MethodHook {
	enabled: bool,
	original_fn_ptr: usize,
	method_body: Vec<u8>,
}

// TODO: Not a fan of using true/false to tell the original function to run or not.
// Should find a better solution.
#[no_mangle]
pub unsafe extern "cdecl" fn exec_mod_fn(stack_ptr: *mut usize) -> bool {
	let hook = *stack_ptr.offset(0) as *const MethodHook;

	if hook.is_null() {
		eprintln!(
			"Hook was not found. At Stack PTR: {:p}. Running original method.",
			stack_ptr
		);
		// Run the original function by returning false
		return false;
	}

	#[allow(unused)]
	let hook = &*hook;

	// Run the hook somehow.

	// Don't run the original function.
	true
}

#[allow(unused)]
unsafe fn wrap_borland_method(
	// pass as: &mut self.original_fn_ptr
	original_fn_ptr: *mut usize,
	// pass as: `&(exec_mod_fn as _)`
	exec_mod_fn: &*const usize,
	// hook info
	lua_hook: &mut MethodHook,
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

		; call DWORD [exec_mod_fn as *const _ as _]
		; add esp, 8 // pop esp, pop lua_hook

		// If function returns true, jump to end
		; test eax, eax
		; jne >end

		; pop eax
		; pop edx
		; pop ecx

		; jmp DWORD [original_fn_ptr as *const _ as _]
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
