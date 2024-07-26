use std::ffi::c_void;

use safer_ffi::{ffi_export, prelude::repr_c};
use winapi::um::{memoryapi::WriteProcessMemory, processthreadsapi::GetCurrentProcess};

use crate::internal::memory::sigscanner::{SigScanner, SigScannerResult};

#[ffi_export]
pub unsafe extern "C" fn scan_memory(signature: repr_c::String) -> *mut c_void {
	let result = SigScanner::new(&signature);
	let result = result.exec();

	if let SigScannerResult::Found(ptr) = result {
		ptr as _
	} else {
		std::ptr::null_mut()
	}
}

#[ffi_export]
pub unsafe extern "C" fn read_bytes(pointer: *const c_void, length: usize) -> repr_c::Vec<u8> {
	let buffer = std::slice::from_raw_parts(pointer as *const u8, length).to_vec();

	buffer.into()
}

#[ffi_export]
pub unsafe extern "C" fn write_bytes(pointer: *const c_void, buffer: repr_c::Vec<u8>) -> bool {
	let result = WriteProcessMemory(
		GetCurrentProcess(),
		pointer as _,
		buffer.as_ptr() as _,
		buffer.len(),
		std::ptr::null_mut(),
	);

	result != 0
}
