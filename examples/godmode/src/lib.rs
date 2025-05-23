#![allow(clippy::missing_safety_doc)]
mod utils;

use safer_ffi::c_char;
use std::{ffi::c_void, ptr::addr_of_mut};
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use utils::{hook_apply, hook_new, hook_revert, scan_memory};

static mut PROC_DMG_STAM_PTR: *mut c_void = std::ptr::null_mut();

// TODO: This godmode makes *everyone* godmode.
// Need to find a way to get the player's motile pointer.

static mut FIRST_RUN: bool = true;

static mut HOOK: Option<safer_ffi::boxed::Box<utils::Hook>> = None;

#[no_mangle]
pub unsafe extern "C" fn enable() -> bool {
	if unsafe { FIRST_RUN } {
		tracing_subscriber::registry()
			.with(
				fmt::layer().with_filter(
					EnvFilter::builder()
						.from_env()
						.unwrap()
						.add_directive("godmode=debug".parse().unwrap()),
				),
			)
			.init();
		unsafe { FIRST_RUN = false };
	}

	let sig = "53 56 48 8D 64 24 D8 48 89 CB 40 30 F6 8B 05 ? ? ? ? 89";
	let ptr = scan_memory(sig.into());
	if ptr.is_null() {
		error!("Failed to find pointer for godmode hook");
		return false;
	}

	println!("Found proc_dmg_and_stamina at: {:p}", ptr);

	PROC_DMG_STAM_PTR = ptr;

	HOOK = Some(hook_new(
		"godmode::proc_dmg_stamina".into(),
		addr_of_mut!(PROC_DMG_STAM_PTR),
		proc_dmg_stam as _,
	));

	let success = hook_apply(HOOK.as_mut().unwrap());

	info!("Hooked proc_dmg_and_stamina: {}", success);

	success
}

#[no_mangle]
pub unsafe extern "C" fn disable() -> bool {
	let success = hook_revert(HOOK.as_mut().unwrap());

	info!("Unhooked proc_dmg_and_stamina: {}", success);

	success
}

#[no_mangle]
unsafe extern "C" fn proc_dmg_stam(motile_ptr: *mut c_void, _: f32, _: f32, _: c_char) -> c_char {
	println!("{:p}", motile_ptr);

	let orig_proc: extern "C" fn(a: *mut c_void, b: f32) -> c_char =
		std::mem::transmute(PROC_DMG_STAM_PTR);
	orig_proc(motile_ptr, 0.0f32)
}
