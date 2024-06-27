use safer_ffi::{derive_ReprC, prelude::repr_c::*};
use std::ffi::c_void;

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct Patch {
    address: *const u8,
    patch_bytes: Vec<u8>,
    original_bytes: Vec<u8>,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct Hook {
    hook_name: String,
    target_fn_ptr: *mut c_void,
    replacement_fn_ptr: *mut c_void,
    hooked: bool,
}

#[link(name = "emf.dll")]
#[allow(unused)]
extern "C" {
    pub(crate) fn scan_memory(signature: String) -> *mut c_void;

    pub(crate) fn read_bytes(pointer: *const c_void, length: usize) -> Vec<u8>;
    pub(crate) fn write_bytes(pointer: *const c_void, buffer: Vec<u8>) -> bool;

    pub(crate) fn patch_new(address: u64, data: Vec<u8>) -> Box<Patch>;
    pub(crate) fn patch_from_signature(signature: String, data: Vec<u8>) -> Option<Box<Patch>>;
    pub(crate) fn patch_offset_pointer(patch: &mut Patch, offset: isize) -> *const u8;
    pub(crate) fn patch_apply(patch: &mut Patch) -> bool;
    pub(crate) fn patch_revert(patch: &mut Patch) -> bool;
    pub(crate) fn patch_is_applied(patch: &Patch) -> bool;
    pub(crate) fn patch_read_current(patch: &Patch) -> Option<Box<Vec<u8>>>;
    pub(crate) fn reassemble_instruction_at_offset(bytes: Vec<u8>, offset: isize) -> Vec<u8>;

    pub(crate) fn hook_new(
        hook_name: String,
        target_fn_ptr: *mut *mut c_void,
        replacement_fn_ptr: *mut c_void,
    ) -> Box<Hook>;
    pub(crate) fn hook_from_signature(
        hook_name: String,
        signature: String,
        replacement_fn_ptr: *mut c_void,
    ) -> Option<Box<Hook>>;
    pub(crate) fn hook_offset_pointer(hook: &mut Hook, offset: isize) -> *mut *mut c_void;
    pub(crate) fn hook_apply(hook: &mut Hook) -> bool;
    pub(crate) fn hook_revert(hook: &mut Hook) -> bool;
    pub(crate) fn hook_is_applied(hook: &Hook) -> bool;
}
