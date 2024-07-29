#![allow(clippy::missing_safety_doc)]

use log::*;
use std::ffi::{c_char, c_void};

use emf_rs::macros::plugin;
use emf_rs::safer_ffi::prelude::char_p;

#[plugin(id = "dev.megu.demo-plugin")]
mod plugin {
	// #[hook_signature] is used to hook a function based on a byte signature, provided as a string.
	// take a look at the register!() macro below to see how it's used.
	//
	// There is also a #[hook_address] attribute that uses a pointer instead of a signature.
	// This is useful when finding the address is not as simple as providing a byte signature.
	// e.g.
	// register!(
	//   let address = { some code }; // however you find your address
	//   address // return the address
	// );
	#[hook_signature]
	#[link_setting("godmode_enabled")]
	extern "C" fn proc_dmg_and_stamina(motile_ptr: *mut c_void, _: f32) -> c_char {
		// the register!() macro returns a signature to the target function when using #[hook_signature]
		register!("53 56 48 8D 64 24 D8 48 89 CB 40 30 F6 8B 05 ?? ?? ?? ?? 89");

		println!("Stopping damage for motile: {:p}", motile_ptr);

		// calling proc_dmg_from_stamina from inside the hook runs the original function
		proc_dmg_and_stamina(motile_ptr, 0.0)
	}

	// #[patch_signature] is used to patch a function based on a byte signature, provided as a string.
	// take a look at the register!() macro below to see how it's used.
	//
	// Just like #[hook_address], there is also a #[patch_address] attribute, used in the same way.
	//
	// The address found at the signature (or address) is passed into the function as the first argument.
	// This way, you can read the data from that address if you need to use it. See the examples below.
	#[patch_signature]
	#[link_setting("godhand_enabled")]
	fn ignore_range_limit_for_placement(_address: *mut u8) -> Vec<u8> {
		// the register!() macro returns a signature to the target memory when using #[patch_signature]
		register!("48 8B 40 10 48 8D 48 20");

		vec![
			0x66_u8, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x0F, 0x1F, 0x84, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x45, 0x31, 0xED,
		]
	}

	#[patch_signature(offset = 0x2)]
	#[link_setting("godhand_enabled")]
	fn ignore_range_limit_for_reach(_address: *mut u8) -> Vec<u8> {
		register!("EB 0E 4C 89 E1");

		vec![
			0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x90, 0x45, 0x31, 0xED,
		]
	}

	#[patch_signature(offset = 0xF)]
	#[link_setting("godhand_enabled")]
	fn ignore_range_limit_for_door_and_lever_reach(address: *mut u8) -> Vec<u8> {
		register!("E9 ?? ?? ?? ?? 48 8B 05 ?? ?? ?? ?? 8B 40 ?? 25 ?? ?? ?? ?? ?? ?? 48 8B 05 ?? ?? ?? ?? 48 8B 40 ?? 48 8B 80 ?? ?? ?? ?? 48 8D");

		let instruction = Memory::read_bytes(address, 0x5);
		Memory::reassemble_at_offset(instruction, 0xF)
	}

	#[patch_signature(offset = 0x19)]
	#[link_setting("godhand_enabled")]
	fn ignore_weight(_address: *mut u8) -> Vec<u8> {
		register!("25 ?? ?? ?? ?? 75 ?? F3 0F ?? ?? ?? ?? ?? ?? 66 0F ?? ?? ?? ?? ?? ?? 7A");

		vec![0xEB]
	}

	#[patch_signature(offset = 0x14)]
	#[link_setting("godhand_enabled")]
	fn interact_while_fallen(_address: *mut u8) -> Vec<u8> {
		register!("74 ?? 48 8B 05 ?? ?? ?? ?? 8B 80 ?? ?? ?? ?? 25 ?? ?? ?? ?? 0F 87 ?? ?? ?? ??");

		vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90]
	}

	#[patch_signature(offset = 0x12)]
	#[link_setting("godhand_enabled")]
	fn dont_interrupt_if_fallen(_address: *mut u8) -> Vec<u8> {
		register!("48 8B 05 ?? ?? ?? ?? 8B 80 ?? ?? ?? ?? 25 ?? ?? ?? ?? ?? ?? E8");

		vec![0xEB]
	}
}

#[no_mangle]
pub unsafe extern "C" fn enable() {
	let mut plugin = plugin::get();
	debug!("Enabling plugin {}", plugin.id);

	let plugin_id = plugin.id.clone();
	for patch in plugin.patches.values_mut() {
		let enabled = patch.is_config_enabled(&plugin_id);
		patch.set_enabled(enabled);
	}

	for hook in plugin.hooks.values_mut() {
		let enabled = hook.is_config_enabled(&plugin_id);
		hook.set_enabled(enabled);
	}

	plugin.on_enable().unwrap();
}

#[no_mangle]
pub unsafe extern "C" fn disable() {
	let mut plugin = plugin::get();
	plugin.on_disable().unwrap();
	debug!("Disabling plugin {}", plugin.id);
}

#[no_mangle]
pub unsafe extern "C" fn on_message(s: char_p::Box, m: char_p::Box) {
	plugin::get().on_message(s, m, |sender, message| {
		debug!("Received message from {}: {}", sender, message);
	});
}

#[no_mangle]
pub unsafe extern "C" fn setting_changed_bool(name: char_p::Box, value: bool) {
	// If the ID of a boolean setting starts with `patch::` or `hook::`,
	// the plugin will automatically enable/disable the corresponding patch/hook.

	// Example:
	//
	// [[setting]]
	// name = "My Setting"
	// id = "patch::my_patch"
	//
	// or
	//
	// [[setting]]
	// name = "My Setting"
	// id = "hook::my_hook"

	plugin::get().on_setting_changed_bool(name, value, |key, value| {
		// Do something with this
		debug!("Setting changed: {} = {}", key, value);
	});
}

#[no_mangle]
pub unsafe extern "C" fn setting_changed_int(name: char_p::Box, value: i32) {
	plugin::get().on_setting_changed_int(name, value, |key, value| {
		// Do something with this
		debug!("Setting changed: {} = {}", key, value);
	});
}

#[no_mangle]
pub unsafe extern "C" fn setting_changed_float(name: char_p::Box, value: f32) {
	plugin::get().on_setting_changed_float(name, value, |key, value| {
		// Do something with this
		debug!("Setting changed: {} = {}", key, value);
	});
}

#[no_mangle]
pub unsafe extern "C" fn setting_changed_string(name: char_p::Box, value: char_p::Box) {
	plugin::get().on_setting_changed_string(name, value, |key, value| {
		// Do something with this
		debug!("Setting changed: {} = {}", key, value);
	});
}
