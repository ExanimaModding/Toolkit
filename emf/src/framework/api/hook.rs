use crate::internal::memory::sigscanner::SigScanner;
use crate::{framework::api::location_is_readwrite, internal::utils::ntdll::NtStatus};
use anyhow::*;
use detours_sys::{
	DetourAttach, DetourDetach, DetourTransactionAbort, DetourTransactionBegin,
	DetourTransactionCommit,
};
use log::*;
use safer_ffi::{derive_ReprC, ffi_export, prelude::repr_c};
use std::ffi::c_void;
use std::result::Result::Ok;
use winapi::shared::ntdef::NTSTATUS;
use winapi::um::processthreadsapi::GetCurrentProcess;

pub trait Hookable {
	/// Apply the hook.
	unsafe fn apply(&mut self) -> Result<()>;
	/// Revert the hook.
	unsafe fn revert(&mut self) -> Result<()>;
	/// Check if the hook is already applied.
	unsafe fn is_applied(&self) -> bool;
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct Hook {
	hook_name: repr_c::String,
	target_fn_ptr: *mut *mut c_void,
	replacement_fn_ptr: *mut c_void,
	hooked: bool,
}

impl Hook {
	/// Create a new function hook, replacing target_fn_ptr with replacement_fn_ptr.
	pub fn new(
		hook_name: String,
		target_fn_ptr: *mut *mut c_void,
		replacement_fn_ptr: *mut c_void,
	) -> Self {
		Self {
			hook_name: hook_name.into(),
			target_fn_ptr,
			replacement_fn_ptr,
			hooked: false,
		}
	}

	/// Get a pointer from a signature, and use that to create a new hook.
	pub unsafe fn from_signature(
		hook_name: String,
		signature: &str,
		replacement_fn_ptr: *mut c_void,
	) -> Result<Self> {
		let result = SigScanner::new(signature);
		let result = result.exec();

		if let crate::internal::memory::sigscanner::SigScannerResult::Found(ptr) = result {
			Ok(Self::new(hook_name, ptr as _, replacement_fn_ptr))
		} else {
			Err(anyhow!("Signature not found."))
		}
	}

	/// Offset the target function pointer by a given amount of bytes.
	pub unsafe fn offset_pointer(&mut self, offset: isize) -> *mut *mut c_void {
		self.target_fn_ptr = self.target_fn_ptr.byte_offset(offset);
		self.target_fn_ptr
	}
}

impl Hookable for Hook {
	unsafe fn apply(&mut self) -> Result<()> {
		if self.hooked {
			return Ok(());
		}

		if self.target_fn_ptr.is_null() || self.replacement_fn_ptr.is_null() {
			return Err(anyhow!("Target or replacement function is null."));
		}

		let proc = GetCurrentProcess();
		let writable = location_is_readwrite(*self.target_fn_ptr as _, proc);

		if writable.is_err() {
			return Err(anyhow!(
				"Target or replacement function is not read-writeable."
			));
		}

		DetourTransactionBegin();

		let result: NTSTATUS = DetourAttach(self.target_fn_ptr, self.replacement_fn_ptr);

		if let NtStatus::Other(status) = NtStatus::from(result) {
			DetourTransactionAbort();
			Err(anyhow!("Failed to attach detour. Status: {:#X}", status))
		} else {
			DetourTransactionCommit();
			self.hooked = true;
			Ok(())
		}
	}

	unsafe fn revert(&mut self) -> Result<()> {
		if !self.hooked {
			return Ok(());
		}

		if self.target_fn_ptr.is_null() || self.replacement_fn_ptr.is_null() {
			return Err(anyhow!("Target or replacement function is null."));
		}

		let proc = GetCurrentProcess();
		let writable = location_is_readwrite(*self.target_fn_ptr as _, proc);

		if writable.is_err() {
			return Err(anyhow!(
				"Target or replacement function is not read-writeable."
			));
		}

		DetourTransactionBegin();
		let result: NTSTATUS = DetourDetach(self.target_fn_ptr, self.replacement_fn_ptr);

		if let NtStatus::Other(status) = NtStatus::from(result) {
			DetourTransactionAbort();
			Err(anyhow!("Failed to detach detour. Status: {:#X}", status))
		} else {
			DetourTransactionCommit();
			self.hooked = false;
			Ok(())
		}
	}

	unsafe fn is_applied(&self) -> bool {
		// TODO: Make this dynamically check, rather than storing a bool.
		self.hooked
	}
}

#[ffi_export]
/// Create a function hook, replacing target_fn_ptr with replacement_fn_ptr.
pub extern "C" fn hook_new(
	hook_name: repr_c::String,
	target_fn_ptr: *mut *mut c_void,
	replacement_fn_ptr: *mut c_void,
) -> repr_c::Box<Hook> {
	Box::new(Hook::new(
		hook_name.into(),
		target_fn_ptr,
		replacement_fn_ptr,
	))
	.into()
}

#[ffi_export]
/// Get a pointer from a signature, and use that to create a new hook.
pub unsafe extern "C" fn hook_from_signature(
	hook_name: repr_c::String,
	signature: repr_c::String,
	replacement_fn_ptr: *mut c_void,
) -> Option<repr_c::Box<Hook>> {
	match Hook::from_signature(hook_name.into(), &signature, replacement_fn_ptr) {
		Ok(hook) => Some(Box::new(hook).into()),
		Err(e) => {
			error!("{:?}", e);
			None
		}
	}
}

#[ffi_export]
/// Offset the target function pointer by the given offset.
pub unsafe extern "C" fn hook_offset_pointer(hook: &mut Hook, offset: isize) -> *mut *mut c_void {
	hook.offset_pointer(offset)
}

#[ffi_export]
/// Apply the hook.
pub unsafe extern "C" fn hook_apply(hook: &mut Hook) -> bool {
	match hook.apply() {
		Ok(_) => true,
		Err(e) => {
			error!("{:?}", e);
			false
		}
	}
}

#[ffi_export]
/// Revert the hook.
pub unsafe extern "C" fn hook_revert(hook: &mut Hook) -> bool {
	match hook.revert() {
		Ok(_) => true,
		Err(e) => {
			error!("{:?}", e);
			false
		}
	}
}

#[ffi_export]
/// Check if the hook is already applied.
pub unsafe extern "C" fn hook_is_applied(hook: &Hook) -> bool {
	hook.is_applied()
}
