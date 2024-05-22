use std::ffi::c_void;

use anyhow::*;
use detours_sys::DetourAttach;
use safer_ffi::{derive_ReprC, prelude::repr_c};
use winapi::shared::ntdef::NTSTATUS;

use crate::internal::utils::ntdll::NtStatus;

use super::location_is_readwrite;

pub trait Hookable {
	unsafe fn hook(&mut self) -> Result<()>;
	unsafe fn unhook(&mut self) -> Result<()>;
	unsafe fn is_hooked(&self) -> bool;
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct Hook {
	hook_name: repr_c::String,
	target_fn_ptr: *mut c_void,
	replacement_fn_ptr: *mut c_void,
}

impl Hookable for Hook {
	unsafe fn hook(&mut self) -> Result<()> {
		let writable = location_is_readwrite(self.target_fn_ptr as _, self.replacement_fn_ptr as _);

		if writable.is_err() {
			return Err(anyhow!(
				"Target or replacement function is not read-writeable."
			));
		}

		let result: NTSTATUS = DetourAttach(&raw mut self.target_fn_ptr, self.replacement_fn_ptr);

		if let NtStatus::Other(status) = NtStatus::from(result) {
			Err(anyhow!("Failed to attach detour. Status: {:#X}", status))
		} else {
			Ok(())
		}
	}

	unsafe fn is_hooked(&self) -> bool {
		// TODO:
		todo!();
	}

	unsafe fn unhook(&mut self) -> Result<()> {
		let writable = location_is_readwrite(self.target_fn_ptr as _, self.replacement_fn_ptr as _);

		if writable.is_err() {
			return Err(anyhow!(
				"Target or replacement function is not read-writeable."
			));
		}

		let result: NTSTATUS = DetourAttach(&raw mut self.target_fn_ptr, self.replacement_fn_ptr);

		if let NtStatus::Other(status) = NtStatus::from(result) {
			Err(anyhow!("Failed to attach detour. Status: {:#X}", status))
		} else {
			Ok(())
		}
	}
}
