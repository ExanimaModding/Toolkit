mod hook;
mod memory;
mod patch;

pub use emf_rs_macros as macros;
pub use emf_sys as sys;
pub use memory::Memory;
pub use once_cell;
pub use safer_ffi;

use anyhow::{anyhow, Result};
use hook::Hookable;
use log::*;
use patch::Patchable;
use safer_ffi::prelude::char_p;
use std::{collections::HashMap, ffi::c_void};

pub struct Plugin {
	pub id: String,
	pub hooks: HashMap<String, Box<dyn Hookable>>,
	pub patches: HashMap<String, Box<dyn Patchable>>,
	pub setting_links: HashMap<String, Vec<String>>,
	pub enabled: bool,
}

unsafe impl Send for Plugin {}
unsafe impl Sync for Plugin {}

impl Plugin {
	pub fn new(id: &str) -> Self {
		Plugin {
			id: id.to_string(),
			hooks: HashMap::new(),
			patches: HashMap::new(),
			setting_links: HashMap::new(),
			enabled: false,
		}
	}

	/// Enable the plugin.
	///
	/// This function should be called after all hooks and patches have been created.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	// TODO: What should this return? Should it break if any fail?
	pub unsafe fn on_enable(&mut self) -> Result<()> {
		self.enabled = true;
		for hook in self.hooks.values_mut() {
			// Pull the latest state from the config file.
			let enabled = hook.is_config_enabled(&self.id);
			hook.set_enabled(enabled);

			if self.enabled && hook.is_enabled() && !hook.is_applied() {
				match hook.apply() {
					Ok(_) => {
						debug!("Applied hook: {}", hook.get_name());
					}
					Err(e) => {
						error!("Failed to apply hook: {:?}", e);
					}
				}
			}
		}

		for patch in self.patches.values_mut() {
			// Pull the latest state from the config file.
			let enabled = patch.is_config_enabled(&self.id);
			patch.set_enabled(enabled);

			if self.enabled && patch.is_enabled() && !patch.is_applied() {
				match patch.apply() {
					Ok(_) => {
						debug!("Applied patch: {}", patch.get_name());
					}
					Err(e) => {
						error!("Failed to apply patch: {:?}", e);
					}
				}
			}
		}

		Ok(())
	}

	/// Disable the plugin.
	///
	/// This function should be called before the plugin is unloaded.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	// TODO: What should this return? Should it break if any fail?
	pub unsafe fn on_disable(&mut self) -> Result<()> {
		self.enabled = false;
		for hook in self.hooks.values_mut() {
			if hook.is_applied() {
				match hook.revert() {
					Ok(_) => {
						debug!("Reverted hook: {}", hook.get_name());
					}
					Err(e) => {
						error!("Failed to revert hook: {:?}", e);
					}
				}
			}
		}

		for patch in self.patches.values_mut() {
			if patch.is_applied() {
				match patch.revert() {
					Ok(_) => {
						debug!("Reverted patch: {}", patch.get_name());
					}
					Err(e) => {
						error!("Failed to revert patch: {:?}", e);
					}
				}
			}
		}

		Ok(())
	}

	/// Handle a message from another plugin.
	pub fn on_message(
		&mut self,
		sender: char_p::Box,
		message: char_p::Box,
		handler: fn(&str, &str),
	) {
		let sender = sender.to_string();
		let message = message.to_string();

		handler(&sender, &message);
	}

	/// Send a message to another plugin.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn send_message(&mut self, target: &str, message: &str) {
		let sender = self.id.to_string();
		let target = target.to_string();
		let message = message.to_string();

		sys::send_message(sender.into(), target.into(), message.into())
	}

	/// Create a function hook
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	///
	/// # Examples
	///
	/// ```
	/// extern "C" fn replacement_fn() {
	///     debug!("Hello, world!");
	/// }
	///
	/// unsafe fn main() {
	///     type MyType = extern "C" fn(u32, u32);
	///     let target_ptr = 0xDEADBEEF as *mut ();
	///     let hook = plugin.create_hook::<MyType>("my_hook", target_ptr, replacement_fn as _);
	/// }
	/// ```
	pub unsafe fn create_hook(
		&mut self,
		hook_name: &str,
		target_fn: *mut *mut c_void,
		replacement_fn: *mut c_void,
	) -> Result<&mut Box<dyn hook::Hookable>> {
		if self.hooks.contains_key(hook_name) {
			return Err(anyhow!("Hook already exists."));
		}

		let hook = hook::Hook::new(hook_name, target_fn, replacement_fn);
		let hook = Box::new(hook);

		self.hooks.insert(hook_name.to_string(), hook);
		Ok(self.hooks.get_mut(hook_name).unwrap())
	}

	/// Create a byte patch
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn create_patch(
		&mut self,
		name: &str,
		address: *const u8,
		patch_bytes: Vec<u8>,
	) -> Result<&mut Box<dyn patch::Patchable>> {
		if self.patches.contains_key(name) {
			return Err(anyhow!("Patch already exists."));
		}

		let patch = patch::Patch::new(name, address, patch_bytes);
		let patch = Box::new(patch);

		self.patches.insert(name.to_string(), patch);
		Ok(self.patches.get_mut(name).unwrap())
	}

	/// Link setting_name to link_to.
	///
	/// e.g. If you link_setting("a", "b"), then when b is enabled/disabled, a will be enabled/disabled as well.
	pub fn link_setting(&mut self, setting_name: &str, link_to: &str) {
		if !self.setting_links.contains_key(link_to) {
			self.setting_links
				.insert(link_to.to_string(), vec![setting_name.to_string()]);
		}

		self.setting_links
			.get_mut(link_to)
			.unwrap()
			.push(setting_name.to_string());

		dbg!(&self.setting_links);
	}
}

impl Plugin {
	/// Read a boolean setting.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn read_setting_bool(&mut self, name: &str) -> Result<bool> {
		let value = sys::get_setting_bool(self.id.clone().into(), name.into());
		match value.found {
			true => Ok(value.value),
			false => Err(anyhow!("Setting not found.")),
		}
	}

	/// Read an integer setting.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn read_setting_int(&mut self, name: &str) -> Result<i64> {
		let value = sys::get_setting_integer(self.id.clone().into(), name.into());
		match value.found {
			true => Ok(value.value),
			false => Err(anyhow!("Setting not found.")),
		}
	}

	/// Read a float setting.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn read_setting_float(&mut self, name: &str) -> Result<f64> {
		let value = sys::get_setting_float(self.id.clone().into(), name.into());
		match value.found {
			true => Ok(value.value),
			false => Err(anyhow!("Setting not found.")),
		}
	}

	/// Read a string setting.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn read_setting_string(&mut self, name: &str) -> Result<String> {
		let value = sys::get_setting_string(self.id.clone().into(), name.into());
		match value.found {
			true => Ok(value.value.to_owned().into()),
			false => Err(anyhow!("Setting not found.")),
		}
	}

	unsafe fn handle_setting_toggle(
		&mut self,
		setting_name: &str,
		value: bool,
		// user-side handler for setting changes
		handler: fn(&str, bool),
	) {
		let name = setting_name.to_string();
		if name.is_empty() {
			return;
		}

		if name.starts_with("patch::") {
			let patch_name = name.strip_prefix("patch::").unwrap();

			if let Some(patch) = self.patches.get_mut(patch_name) {
				patch.set_enabled(value);

				if value && !patch.is_applied() {
					patch.apply().unwrap();
					debug!("Applied patch.");
				} else if !value && patch.is_applied() {
					patch.revert().unwrap();
					debug!("Reverted patch.");
				}
				handler(&name, value);
			}
		} else if name.starts_with("hook::") {
			let hook_name = name.strip_prefix("hook::").unwrap();

			debug!("Hook name: {}", hook_name);

			if let Some(hook) = self.hooks.get_mut(hook_name) {
				hook.set_enabled(value);

				if value && !hook.is_applied() {
					hook.apply().unwrap();
					debug!("Applied hook.");
				} else if !value && hook.is_applied() {
					hook.revert().unwrap();
					debug!("Reverted patch.");
				}
				handler(&name, value);
			}
		} else if let Some(linked_settings) = self.setting_links.get_mut(setting_name).cloned() {
			dbg!(&linked_settings);
			for linked_setting in linked_settings {
				// Prevent infinite loops.
				if linked_setting == setting_name {
					continue;
				}

				if linked_setting.starts_with("patch::") || linked_setting.starts_with("hook::") {
					self.handle_setting_toggle(&linked_setting, value, handler);
				}
			}
		}
	}

	/// Handle a boolean setting change.
	///
	/// If the setting name starts with `patch::`, the patch will be toggled.
	///
	/// If the setting name starts with `hook::`, the hook will be toggled.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	pub unsafe fn on_setting_changed_bool(
		&mut self,
		name: char_p::Box,
		value: bool,
		handler: fn(&str, bool),
	) {
		let name = name.to_string();

		self.handle_setting_toggle(&name, value, handler);

		handler(&name, value);
	}

	/// Handle an integer setting change.
	pub fn on_setting_changed_int(
		&mut self,
		name: char_p::Box,
		value: i32,
		handler: fn(&str, i32),
	) {
		let name = name.to_string();

		if name.is_empty() {
			return;
		}

		handler(&name, value);
	}

	/// Handle a float setting change.
	pub fn on_setting_changed_float(
		&mut self,
		name: char_p::Box,
		value: f32,
		handler: fn(&str, f32),
	) {
		let name = name.to_string();

		if name.is_empty() {
			return;
		}

		handler(&name, value);
	}

	/// Handle a string setting change.
	pub fn on_setting_changed_string(
		&mut self,
		name: char_p::Box,
		value: char_p::Box,
		handler: fn(&str, &str),
	) {
		let name = name.to_string();
		let value = value.to_string();

		if name.is_empty() {
			return;
		}

		handler(&name, &value);
	}
}

#[macro_export]
macro_rules! create_patch {
	($plugin:expr, $name:expr, $address:expr, $bytes:expr) => {
		if $address.is_null() {
			eprintln!("Address for patch {} is null.", $name);
		} else {
			match $plugin.create_patch($name, $address, $bytes) {
				Ok(_) => {}
				Err(e) => {
					eprintln!("Failed to create patch: {:?}", e);
				}
			}
		}
	};
}

#[macro_export]
macro_rules! make_hook {
	(fn $name:ident($($arg_name:ident: $arg_type:ty),*) -> $ret_type:ty $body:block) => {
		pub(crate) mod $name {
			use super::*;

			pub(crate) type __FnSig = extern "C" fn($($arg_name: $arg_type),*) -> $ret_type;
			pub(crate) static mut TARGET_FN: *mut std::ffi::c_void = 0 as *mut std::ffi::c_void;

			#[no_mangle]
			pub(crate) unsafe extern "C" fn func($($arg_name: $arg_type),*) -> $ret_type {
				let $name = std::mem::transmute::<*mut std::ffi::c_void, __FnSig>(TARGET_FN);
				$body
			}

			pub(crate) unsafe fn set_ptr(new_ptr: *mut std::ffi::c_void) {
				TARGET_FN = new_ptr;
			}
		}
	};
}

#[macro_export]
macro_rules! register_hook {
	($plugin:ident, $name:ident, $address:ident) => {
		unsafe {
			$name::set_ptr($address as *mut std::ffi::c_void);
			let target_ptr = std::ptr::addr_of_mut!($name::TARGET_FN);
			let hook = $plugin.create_hook(stringify!($name), target_ptr, $name::func as _);
			match hook {
				Ok(_) => Some(hook),
				Err(e) => {
					eprintln!("Failed to create hook {}: {:?}", stringify!($name), e);
					None
				}
			}
		}
	};
}
