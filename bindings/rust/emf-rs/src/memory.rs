use std::ffi::c_void;

use crate::sys;

pub struct Memory;

impl Memory {
	/// Scan memory for a signature.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn sig_scan(sig: &str) -> *mut c_void {
		sys::scan_memory(sig.into())
	}

	/// Read bytes from memory.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn read_bytes(address: *const u8, size: usize) -> Vec<u8> {
		sys::read_bytes(address as _, size).into()
	}

	/// Write bytes to memory.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary and writes directly to memory.
	pub unsafe fn write_bytes(address: *mut u8, bytes: Vec<u8>) {
		sys::write_bytes(address as _, bytes.into());
	}

	/// Reassemble an instruction at an offset.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn reassemble_at_offset(bytes: Vec<u8>, offset: usize) -> Vec<u8> {
		sys::reassemble_instruction_at_offset(bytes.into(), offset).into()
	}
}
