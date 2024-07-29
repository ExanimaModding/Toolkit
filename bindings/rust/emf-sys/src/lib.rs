use safer_ffi::{boxed::Box, prelude::*, String, Vec};
use std::ffi::c_void;

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct PatchRaw {
	pub address: *const u8,
	pub patch_bytes: Vec<u8>,
	pub original_bytes: Vec<u8>,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct HookRaw {
	pub hook_name: String,
	pub target_fn_ptr: *mut *mut c_void,
	pub replacement_fn_ptr: *mut c_void,
	pub hooked: bool,
}

#[link(name = "emf.dll")]
extern "C" {
	/// Send a message to another plugin.
	///
	/// target_id: The ID of the target plugin.
	///
	/// message: The message to send.
	pub fn send_message(sender_id: String, target_id: String, message: String);

	/// Scan the memory for a signature.
	///
	/// signature: The signature to scan for.
	///
	/// Returns the address of the first occurrence of the signature.
	pub fn scan_memory(signature: String) -> *mut c_void;

	/// Read bytes from a memory address.
	///
	/// pointer: The memory address to read from.
	///
	/// length: The number of bytes to read.
	///
	/// Returns the bytes read.
	pub fn read_bytes(pointer: *const c_void, length: usize) -> Vec<u8>;

	/// Write bytes to a memory address.
	///
	/// pointer: The memory address to write to.
	///
	/// buffer: The bytes to write.
	///
	/// Returns true if the write was successful.
	pub fn write_bytes(pointer: *const c_void, buffer: Vec<u8>) -> bool;

	/// Create a new patch from a pointer.
	///
	/// address: The address to patch.
	///
	/// data: The data to write to the address.
	///
	/// Returns the patch.
	pub fn patch_new(address: u64, data: Vec<u8>) -> Box<PatchRaw>;

	/// Create a new patch from a signature.
	///
	/// signature: The signature to find. The pointer of the signature will be used as the destination.
	///
	/// data: The data to write to the address.
	///
	/// Returns the patch.
	pub fn patch_from_signature(signature: String, data: Vec<u8>) -> Option<Box<PatchRaw>>;

	/// Offset the pointer in a patch.
	///
	/// patch: The patch to offset.
	///
	/// offset: The offset to apply.
	///
	/// Returns the offset pointer.
	pub fn patch_offset_pointer(patch: &mut PatchRaw, offset: isize) -> *const u8;

	/// Apply a patch.
	///
	/// patch: The patch to apply.
	///
	/// Returns true if the patch was applied.
	pub fn patch_apply(patch: &mut PatchRaw) -> bool;

	/// Revert a patch.
	///
	/// patch: The patch to revert.
	///
	/// Returns true if the patch was reverted.
	pub fn patch_revert(patch: &mut PatchRaw) -> bool;

	/// Check if a patch is applied.
	///
	/// patch: The patch to check.
	///
	/// Returns true if the patch is applied.
	pub fn patch_is_applied(patch: &PatchRaw) -> bool;

	/// Read the current bytes at the patch address.
	///
	/// patch: The patch to read.
	///
	/// Returns the current bytes at the patch address.
	pub fn patch_read_current(patch: &PatchRaw) -> Option<Box<Vec<u8>>>;

	/// Reassemble an instruction at an offset.
	///
	/// bytes: The bytes to reassemble.
	///
	/// offset: The offset to reassemble at.
	///
	/// Returns the reassembled instruction.
	pub fn reassemble_instruction_at_offset(bytes: Vec<u8>, offset: usize) -> Vec<u8>;

	/// Create a new function hook.
	///
	/// hook_name: The name of the hook.
	///
	/// target_fn_ptr: The pointer to the target function.
	///
	/// replacement_fn_ptr: The pointer to the replacement function.
	///
	/// Returns the hook.
	///
	/// # Example
	///
	/// ```
	/// let ptr = 0xDEADBEEF as *mut c_void;
	/// fn my_func() {}
	/// let hook = hook_new("my_hook".to_string().into(), &raw mut ptr, my_func as _);
	/// ```
	pub fn hook_new(
		hook_name: String,
		target_fn_ptr: *mut *mut c_void,
		replacement_fn_ptr: *const c_void,
	) -> Box<HookRaw>;

	/// Create a new function hook from a signature.
	///
	/// hook_name: The name of the hook.
	///
	/// signature: The signature to find. The pointer of the signature will be used as the target function.
	///
	/// replacement_fn_ptr: The pointer to the replacement function.
	///
	/// Returns the hook.
	pub fn hook_from_signature(
		hook_name: String,
		signature: String,
		replacement_fn_ptr: *mut c_void,
	) -> Option<Box<HookRaw>>;

	/// Offset the pointer in a hook.
	///
	/// hook: The hook to offset.
	///
	/// offset: The offset to apply.
	///
	/// Returns the offset pointer.
	pub fn hook_offset_pointer(hook: &mut HookRaw, offset: isize) -> *mut *mut c_void;

	/// Apply a hook.
	///
	/// hook: The hook to apply.
	///
	/// Returns true if the hook was applied.
	pub fn hook_apply(hook: &mut HookRaw) -> bool;

	/// Revert a hook.
	///
	/// hook: The hook to revert.
	///
	/// Returns true if the hook was reverted.
	pub fn hook_revert(hook: &mut HookRaw) -> bool;

	/// Check if a hook is applied.
	///
	/// hook: The hook to check.
	///
	/// Returns true if the hook is applied.
	pub fn hook_is_applied(hook: &HookRaw) -> bool;

	/// Get a setting as a boolean.
	pub fn get_setting_bool(
		id: repr_c::String,
		key: repr_c::String,
	) -> Box<emf_types::ffi::GetSettingReturnValue<bool>>;

	/// Get a setting as a string.
	pub fn get_setting_string(
		id: repr_c::String,
		key: repr_c::String,
	) -> Box<emf_types::ffi::GetSettingReturnValue<repr_c::String>>;

	/// Get a setting as an integer.
	pub fn get_setting_integer(
		id: repr_c::String,
		key: repr_c::String,
	) -> Box<emf_types::ffi::GetSettingReturnValue<i64>>;

	/// Get a setting as a float.
	pub fn get_setting_float(
		id: repr_c::String,
		key: repr_c::String,
	) -> Box<emf_types::ffi::GetSettingReturnValue<f64>>;
}
