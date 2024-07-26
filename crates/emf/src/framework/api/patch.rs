use anyhow::*;
use log::*;
use safer_ffi::{derive_ReprC, ffi_export, prelude::repr_c};
use std::result::Result::Ok;
use winapi::um::{
	errhandlingapi::GetLastError, memoryapi::WriteProcessMemory,
	processthreadsapi::GetCurrentProcess,
};

use crate::internal::memory::sigscanner::{SigScanner, SigScannerResult};

use super::location_is_readwrite;

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
	/// Create a new byte patch at the given address
	pub fn new(address: u64, data: Vec<u8>) -> Self {
		Self {
			address: address as *const u8,
			patch_bytes: data.into(),
			original_bytes: vec![].into(),
		}
	}

	/// Get a pointer from a signature, and use that to create a new byte patch.
	pub unsafe fn from_signature(signature: &str, data: Vec<u8>) -> Result<Self> {
		let result = SigScanner::new(signature);
		let result = result.exec();

		if let SigScannerResult::Found(ptr) = result {
			Ok(Self::new(ptr as _, data))
		} else {
			Err(anyhow!("Signature not found."))
		}
	}

	/// Offset the patch address by a given amount of bytes.
	///
	/// Returns the new address.
	pub unsafe fn offset_pointer(&mut self, offset: isize) -> *const u8 {
		self.address = self.address.byte_offset(offset);
		self.address
	}
}

impl Patchable<Vec<u8>> for Patch {
	/// Apply the patch to the memory location.
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

	/// Revert the patch from the memory location.
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

	/// Check if the patch is already applied.
	unsafe fn is_applied(&self) -> bool {
		let proc = GetCurrentProcess();
		if location_is_readwrite(self.address as _, proc).is_err() {
			return false;
		}

		*self.read_current().unwrap() == *self.patch_bytes
	}

	/// Read the current bytes at the memory location.
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
/// Create a new byte patch at the given address
pub extern "C" fn patch_new(address: u64, data: repr_c::Vec<u8>) -> repr_c::Box<Patch> {
	Box::new(Patch::new(address, data.into())).into()
}

#[ffi_export]
/// Get a pointer from a signature, and use that to create a new byte patch.
pub unsafe extern "C" fn patch_from_signature(
	signature: repr_c::String,
	data: repr_c::Vec<u8>,
) -> Option<repr_c::Box<Patch>> {
	match Patch::from_signature(&signature, data.into()) {
		std::result::Result::Ok(patch) => Some(Box::new(patch).into()),
		Err(e) => {
			error!("{:?}", e);
			None
		}
	}
}

#[ffi_export]
/// Offset the patch destination address by a given amount of bytes.
pub unsafe extern "C" fn patch_offset_pointer(patch: &mut Patch, offset: isize) -> *const u8 {
	patch.offset_pointer(offset)
}

#[ffi_export]
/// Apply the patch to the memory location.
pub unsafe extern "C" fn patch_apply(patch: &mut Patch) -> bool {
	let result = patch.apply();
	result.is_ok()
}

#[ffi_export]
/// Revert the patch at the memory location.
pub unsafe extern "C" fn patch_revert(patch: &mut Patch) -> bool {
	patch.revert().is_ok()
}

#[ffi_export]
/// Check if the patch is already applied.
pub unsafe extern "C" fn patch_is_applied(patch: &Patch) -> bool {
	patch.is_applied()
}

#[ffi_export]
/// Read the current bytes at the memory location.
pub unsafe extern "C" fn patch_read_current(patch: &Patch) -> Option<repr_c::Vec<u8>> {
	match patch.read_current() {
		std::result::Result::Ok(data) => Some(data.into()),
		Err(_) => None,
	}
}

#[ffi_export]
/// Reassemble an instruction at the given offset.
///
/// This can be useful if you need to copy an instruction such as a jmp
/// And increment/decrement the operands by the offset.
pub unsafe extern "C" fn reassemble_instruction_at_offset(
	bytes: repr_c::Vec<u8>,
	offset: usize,
) -> Option<repr_c::Vec<u8>> {
	let bytes: Vec<u8> = bytes.clone().into();

	let disassembled = libmem::disassemble(bytes.as_ptr() as _)?;

	let asm = format!("{} {}", disassembled.mnemonic, disassembled.op_str);

	libmem::assemble_ex(&asm, libmem::Arch::X86, offset).map(|bytes| bytes.into())
}
