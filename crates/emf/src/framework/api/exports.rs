use std::ffi::c_void;

use safer_ffi::{ffi_export, prelude::repr_c};
use winapi::um::{memoryapi::WriteProcessMemory, processthreadsapi::GetCurrentProcess};

use crate::{
	internal::memory::sigscanner::{SigScanner, SigScannerResult},
	plugins::manager::{PluginManager, PluginMessage},
};

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

#[ffi_export]
pub unsafe extern "C" fn send_message(
	sender: repr_c::String,
	target: repr_c::String,
	message: repr_c::String,
) -> bool {
	let sender = sender.to_string();
	let target = target.to_string();
	let message = message.to_string();

	PluginManager::send_message(target.as_str(), PluginMessage::Message(sender, message));

	true
}
