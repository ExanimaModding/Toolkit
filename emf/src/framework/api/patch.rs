use anyhow::*;
use libmem_sys::{lm_inst_t, LM_AssembleEx, LM_Disassemble, LM_FreePayload, LM_ARCH_X86, LM_FALSE};
use log::*;
use safer_ffi::{derive_ReprC, ffi_export, prelude::repr_c};
use winapi::um::{
	errhandlingapi::GetLastError, memoryapi::WriteProcessMemory,
	processthreadsapi::GetCurrentProcess,
};

use crate::internal::memory::sigscanner::{SigScanner, SigScannerResult};

use super::location_is_readwrite;

// TODO: Standardise all the functions in here.
// Some stuff use different pointer types, unnecessary .to_owned(), etc.

pub trait Patchable<T> {
	unsafe fn apply(&mut self) -> Result<()>;
	unsafe fn revert(&mut self) -> Result<()>;
	unsafe fn read_current(&self) -> Result<T>;
	unsafe fn is_applied(&self) -> bool;
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct Patch {
	address: *const u8,
	patch_bytes: repr_c::Vec<u8>,
	original_bytes: repr_c::Vec<u8>,
}

impl Patch {
	pub fn new(address: u64, data: Vec<u8>) -> Self {
		Self {
			address: address as *const u8,
			patch_bytes: data.into(),
			original_bytes: vec![].into(),
		}
	}

	pub unsafe fn from_signature(signature: &str, data: Vec<u8>) -> Result<Self> {
		let result = SigScanner::new(signature);
		let result = result.exec();

		if let SigScannerResult::Found(ptr) = result {
			Ok(Self::new(ptr as _, data))
		} else {
			Err(anyhow!("Signature not found."))
		}
	}

	pub unsafe fn from_signature_offset(
		signature: &str,
		offset: isize,
		data: Vec<u8>,
	) -> Result<Self> {
		let result = SigScanner::new(signature);
		let result = result.exec();

		if let SigScannerResult::Found(ptr) = result {
			Ok(Self::new((ptr as isize + offset) as _, data))
		} else {
			Err(anyhow!("Signature not found."))
		}
	}
}

impl Patchable<Vec<u8>> for Patch {
	unsafe fn apply(&mut self) -> Result<()> {
		// If the patch is already applied, return early.
		if self.is_applied() {
			return Ok(());
		}

		let proc = GetCurrentProcess();
		if location_is_readwrite(self.address as _, proc).is_err() {
			return Err(anyhow!(
				"Memory location {:p} is not writeable.",
				self.address
			));
		}

		let length = self.patch_bytes.len();

		let original_bytes: &[u8] = std::slice::from_raw_parts(self.address, length);
		self.original_bytes = original_bytes.to_vec().into();

		let result = WriteProcessMemory(
			proc,
			self.address as _,
			self.patch_bytes.as_ptr() as _,
			length,
			std::ptr::null_mut(),
		);

		if result == 0 {
			return Err(anyhow!(
				"WriteProcessMemory failed on memory location {:p}. Error: {}",
				self.address,
				GetLastError()
			));
		}

		match self.is_applied() {
			true => Ok(()),
			false => Err(anyhow!("Patch failed to apply.")),
		}
	}

	unsafe fn revert(&mut self) -> Result<()> {
		// If the patch is not applied, return early.
		if !self.is_applied() {
			return Ok(());
		}

		let proc = GetCurrentProcess();
		if location_is_readwrite(self.address as _, proc).is_err() {
			return Err(anyhow!(
				"Memory location {:p} is not writeable.",
				self.address
			));
		}

		let length = self.original_bytes.len();

		let result = WriteProcessMemory(
			proc,
			self.address as _,
			self.original_bytes.as_ptr() as _,
			length,
			std::ptr::null_mut(),
		);

		if result == 0 {
			return Err(anyhow!(
				"WriteProcessMemory failed on memory location {:p}. Error: {}",
				self.address,
				GetLastError()
			));
		}

		Ok(())
	}

	unsafe fn is_applied(&self) -> bool {
		let proc = GetCurrentProcess();
		if location_is_readwrite(self.address as _, proc).is_err() {
			return false;
		}

		*self.read_current().unwrap() == *self.patch_bytes
	}

	unsafe fn read_current(&self) -> Result<Vec<u8>> {
		let proc = GetCurrentProcess();
		if location_is_readwrite(self.address as _, proc).is_err() {
			return Err(anyhow!(
				"Memory location {:p} is not writeable.",
				self.address
			));
		}

		let length = self.patch_bytes.len();

		let original_bytes: &[u8] = std::slice::from_raw_parts(self.address, length);
		Ok(original_bytes.to_vec())
	}
}

#[ffi_export]
pub extern "C" fn patch_new(address: u64, data: repr_c::Vec<u8>) -> repr_c::Box<Patch> {
	Box::new(Patch::new(address, data.into())).into()
}

#[ffi_export]
pub extern "C" fn patch_from_signature(
	signature: repr_c::String,
	data: repr_c::Vec<u8>,
) -> Option<repr_c::Box<Patch>> {
	match unsafe { Patch::from_signature(&signature, data.into()) } {
		std::result::Result::Ok(patch) => Some(Box::new(patch).into()),
		Err(e) => {
			error!("{:?}", e);
			None
		}
	}
}

#[ffi_export]
pub extern "C" fn patch_from_signature_offset(
	signature: repr_c::String,
	offset: isize,
	data: repr_c::Vec<u8>,
) -> Option<repr_c::Box<Patch>> {
	match unsafe { Patch::from_signature_offset(&signature, offset, data.to_owned().into()) } {
		std::result::Result::Ok(patch) => Some(Box::new(patch).into()),
		Err(_) => None,
	}
}

#[ffi_export]
pub unsafe extern "C" fn patch_apply(patch: &mut Patch) -> bool {
	patch.apply().is_ok()
}

#[ffi_export]
pub unsafe extern "C" fn patch_revert(patch: &mut Patch) -> bool {
	patch.revert().is_ok()
}

#[ffi_export]
pub unsafe extern "C" fn patch_is_applied(patch: &Patch) -> bool {
	patch.is_applied()
}

#[ffi_export]
pub unsafe extern "C" fn patch_read_current(patch: &Patch) -> Option<repr_c::Vec<u8>> {
	match patch.read_current() {
		std::result::Result::Ok(data) => Some(data.into()),
		Err(_) => None,
	}
}

#[ffi_export]
pub unsafe extern "C" fn reassemble_instruction_at_offset(
	bytes: repr_c::Vec<u8>,
	offset: isize,
) -> Option<repr_c::Vec<u8>> {
	let bytes: Vec<u8> = bytes.clone().into();
	let mut result: lm_inst_t = std::mem::zeroed();
	if LM_Disassemble(bytes.as_ptr() as _, &raw mut result) == LM_FALSE {
		return None;
	}

	let mnemonic = std::ffi::CStr::from_ptr(result.mnemonic.as_ptr())
		.to_str()
		.unwrap()
		.to_owned();

	let ops = std::ffi::CStr::from_ptr(result.op_str.as_ptr())
		.to_str()
		.unwrap()
		.to_owned();

	let asm = std::ffi::CString::new(format!("{} {}", mnemonic, ops)).unwrap();

	let mut reassembled: *mut u8 = std::ptr::null_mut();
	if LM_AssembleEx(
		asm.as_ptr(),
		LM_ARCH_X86,
		64,
		offset as _,
		&raw mut reassembled,
	) == 0
	{
		return None;
	}

	let bytes: Vec<u8> = std::slice::from_raw_parts(reassembled, bytes.len())
		.to_owned()
		.to_vec();

	LM_FreePayload(reassembled);

	Some(bytes.into())
}
