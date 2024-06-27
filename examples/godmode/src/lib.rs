#![feature(raw_ref_op)]
#![allow(clippy::missing_safety_doc)]
mod utils;

use log::*;
use safer_ffi::c_char;
use std::ffi::c_void;
use utils::{hook_apply, hook_new, scan_memory};

static mut PROC_DMG_STAM_PTR: *mut c_void = std::ptr::null_mut();

// TODO: This godmode makes *everyone* godmode.
// Need to find a way to get the player's motile pointer.

#[no_mangle]
pub unsafe extern "C" fn enable() -> bool {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let sig = "53 56 48 8D 64 24 D8 48 89 CB 40 30 F6 8B 05 ? ? ? ? 89";
    let ptr = scan_memory(sig.into());
    if ptr.is_null() {
        error!("Failed to find pointer for godmode hook");
        return false;
    }

    println!("Found proc_dmg_and_stamina at: {:p}", ptr);

    PROC_DMG_STAM_PTR = ptr;

    let mut hook = hook_new(
        "godmode::proc_dmg_stamina".into(),
        &raw mut PROC_DMG_STAM_PTR,
        proc_dmg_stam as _,
    );

    let success = hook_apply(&mut hook);

    info!("Hooked proc_dmg_and_stamina: {}", success);

    success
}

#[no_mangle]
pub extern "C" fn disable() -> bool {
    true
}

#[no_mangle]
unsafe extern "C" fn proc_dmg_stam(motile_ptr: *mut c_void, _b: f32) -> c_char {
    println!("{:p}", motile_ptr);

    let orig_proc: extern "C" fn(a: *mut c_void, b: f32) -> c_char =
        std::mem::transmute(PROC_DMG_STAM_PTR);
    orig_proc(motile_ptr, 0.0f32)
}
