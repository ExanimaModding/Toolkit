use detours_sys::{DetourAttach, DetourDetach, DetourTransactionBegin, DetourTransactionCommit};

use super::{HookImpl, HookType};

/// A hook that detours a function.
/// This *replaces* the original function.
/// You can call the original function from the detour.
#[derive(Default, Debug)]
pub struct DetourHook {
	active: bool,
	original_ptr: usize,
	detour_ptr: usize,
}

impl DetourHook {
	pub fn new(original_ptr: usize, detour_ptr: usize) -> Self {
		Self {
			active: false,
			original_ptr,
			detour_ptr,
		}
	}
}

impl HookImpl for DetourHook {
	fn get_type(&self) -> HookType {
		HookType::Detour
	}

	fn get_original_ptr(&self) -> usize {
		self.original_ptr
	}

	fn get_detour_ptr(&self) -> usize {
		self.detour_ptr
	}

	unsafe fn attach(&mut self) -> bool {
		DetourTransactionBegin();
		let result = DetourAttach(&mut self.original_ptr as *mut _ as _, self.detour_ptr as _);
		DetourTransactionCommit();
		dbg!(result);

		self.active = result == 0;
		self.active
	}

	unsafe fn detach(&mut self) -> bool {
		DetourTransactionBegin();
		let result = DetourDetach(&mut self.original_ptr as *mut _ as _, self.detour_ptr as _);
		DetourTransactionCommit();
		dbg!(result);

		self.active = result == 0;
		self.active
	}
}
