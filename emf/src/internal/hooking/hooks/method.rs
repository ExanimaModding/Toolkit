use detours_sys::{DetourAttach, DetourDetach, DetourTransactionBegin, DetourTransactionCommit};

use super::{HookImpl, HookType};

/// TODO: Improve this
/// A hook that makes a trampoline for a function.
/// This *wraps* the original function.
/// Currently, if it returns true, the original function will be called.
/// If it returns false, the original function will not be called.
#[derive(Default, Debug)]
pub struct MethodHook {
	active: bool,
	original_ptr: usize,
	method_body: Vec<u8>,
}

impl HookImpl for MethodHook {
	fn get_type(&self) -> HookType {
		HookType::Method
	}

	fn get_original_ptr(&self) -> usize {
		self.original_ptr
	}

	fn get_detour_ptr(&self) -> usize {
		self.method_body.as_ptr() as usize
	}

	unsafe fn attach(&mut self) -> bool {
		DetourTransactionBegin();
		let result = DetourAttach(
			&mut self.original_ptr as *mut _ as _,
			self.method_body.as_mut_ptr() as _,
		);
		DetourTransactionCommit();

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

		self.active = result == 0;
		self.active
	}
}
