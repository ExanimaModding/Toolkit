// All exports in this file should be added to `emf/lib.def` or they won't be available to the plugins.

use std::ffi::c_void;

use detours_sys::{
	DetourAttach, DetourTransactionAbort, DetourTransactionBegin, DetourTransactionCommit,
};
use log::*;

use emf_types::{
	hooks::HookFunctionPayloadRaw,
	patches::{PatchMemoryPayloadRaw, ReadMemoryPayloadRaw},
	plugin::PluginMessageRaw,
};
use safer_ffi::{ffi_export, prelude::repr_c};
use winapi::{
	shared::ntdef::NT_SUCCESS,
	um::{memoryapi::WriteProcessMemory, processthreadsapi::GetCurrentProcess},
};

use crate::internal::{
	memory::sigscanner::{SigScanner, SigScannerResult},
	plugins::plugin_manager,
};

#[no_mangle]
/// Send messages between plugins.
pub unsafe extern "C" fn send_plugin_message(message: *const PluginMessageRaw) {
	// info!("Received message: {:#?}", (*message).serialize());

	plugin_manager::PluginManager.send_message(message);
}

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

// TODO: Replace all of these with #[ffi_export]

#[no_mangle]
/// Hook a function.
pub unsafe extern "C" fn hook_function(payload: *const HookFunctionPayloadRaw) -> bool {
	let payload = (*payload).serialize();

	DetourTransactionBegin();
	let status = DetourAttach(
		payload.target_function_ptr,
		payload.replacement_function_ptr,
	);

	if !NT_SUCCESS(status) {
		DetourTransactionAbort();
		error!("Failed to hook function. Status: {:#?}", status);
		false
	} else {
		DetourTransactionCommit();
		true
	}
}

#[no_mangle]
/// Read bytes
pub unsafe extern "C" fn read_memory(payload: *const ReadMemoryPayloadRaw) -> *mut u8 {
	let payload = &*payload;

	let buffer =
		std::slice::from_raw_parts(payload.target_address as *const u8, payload.read_length)
			.to_vec();

	let (ptr, _, _) = Vec::into_raw_parts(buffer);

	ptr
}

#[no_mangle]
/// Replace bytes
pub unsafe extern "C" fn write_memory(payload: *const PatchMemoryPayloadRaw) -> bool {
	let payload = (*payload).serialize();

	info!("Target Address: {:p}", payload.target_address);
	info!("Replacement Bytes: {:?}", payload.replacement_bytes);

	let result = WriteProcessMemory(
		GetCurrentProcess(),
		payload.target_address as _,
		payload.replacement_bytes.as_ptr() as _,
		payload.replacement_bytes.len(),
		std::ptr::null_mut(),
	);

	result != 0
}
