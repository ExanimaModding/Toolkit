use std::ffi::c_void;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use std::arch::asm;

pub fn _start(_target_fn: *const c_void) {
	let mut ops = dynasmrt::x86::Assembler::new().unwrap();

	let mut eax = 0_u32;
	let mut ebx = 0_u32;
	let mut ecx = 0_u32;
	let mut edx = 0_u32;

	let _eax_ptr = &mut eax as *mut u32;

	dynasm!(ops
					; .arch x86
					; mov DWORD [&mut eax as *mut _ as _], eax
					; mov DWORD [&mut ebx as *mut _ as _], ebx
					; mov DWORD [&mut ecx as *mut _ as _], ecx
					; mov DWORD [&mut edx as *mut _ as _], edx
					; ret
	);

	let buf = ops.finalize().unwrap();

	let hello_fn: extern "C" fn() -> bool = unsafe { std::mem::transmute(buf.as_ptr()) };

	dbg!(eax, ebx, ecx, edx);

	hello_fn();

	dbg!(eax, ebx, ecx, edx);
}

static mut FUNC_PTR: *const () = b2f as _;

#[allow(unused)]
#[naked]
pub unsafe extern "fastcall" fn to_borland() {
	asm!("push ecx", "mov ecx, eax", options(noreturn));
}

#[naked]
#[no_mangle]
pub unsafe extern "fastcall" fn borland_to_fastcall() {
	asm!(
		"push ecx",
		"mov ecx, eax", // bl->fc: ecx = eax
		// "mov edx, edx" // bl->fc: edx = edx

		"jmp [{symbol}]",
		symbol = sym FUNC_PTR,
		options(noreturn),
	);
}

#[no_mangle]
pub extern "fastcall" fn b2f() {
	dbg!("owo");
}

pub unsafe fn wrap_borland(hook_fn: &*const c_void, original: &*mut c_void) -> Vec<u8> {
	let mut ops = dynasmrt::x86::Assembler::new().unwrap();

	// dynasm!(ops
	//   ; .arch x86
	//   // ; pushad
	//   ; push ecx
	//   ; push edx
	//   ; push eax
	//   ; call DWORD [hook_fn as *const _ as _]
	//   // ; popad
	//   ; pop eax
	//   ; pop edx
	//   ; pop ecx
	//   ; jmp DWORD [original as *const _ as _]
	//   ; retn 8
	// );

	dynasm!(ops
	  ; .arch x86

	  // Push args to stack for cdecl call
	  ; push ecx
	  ; push edx
	  ; push eax

	  // Call hook function
	  ; call DWORD [hook_fn as *const _ as _]

	  // If function returns true, jump to end
	  ; test eax, eax
	  ; jne >end

	  // Clear the stack
	  ; pop eax
	  ; pop edx
	  ; pop ecx

	  // Run original hook
	  ; jmp DWORD [original as *const _ as _]
	  ; retn 8

	  // Cleanup fn
	  ; end:
	  ; pop eax
	  ; pop edx
	  ; pop ecx
	  ; retn 8
	);

	let buf = ops.finalize().unwrap();

	buf.to_vec()
}
