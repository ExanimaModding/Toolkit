use std::ffi::c_void;

/// A function hook that redirects function calls to a replacement function.
pub struct Hook {
	hook_name: String,
	raw: safer_ffi::boxed::Box<crate::sys::HookRaw>,

	/// Whether the hook should be used when the plugin is enabled
	enabled: bool,
}

impl Hook {
	/// Create a new function hook.
	///
	/// # Examples
	///
	/// ```
	/// extern "C" fn replacement_fn() {
	///     println!("Hello, world!");
	/// }
	///
	/// unsafe fn enable() {
	///     use std::ptr::addr_of_mut;
	///     let mut target_fn = 0xDEADBEEF as *mut c_void;
	///     let hook = Hook::new("my_hook", addr_of_mut!(target_fn), replacement_fn as _);
	/// }
	/// ```
	pub unsafe fn new(
		hook_name: &str,
		target_fn: *mut *mut c_void,
		replacement_fn: *mut c_void,
	) -> Self {
		let hook_name = hook_name.to_string();
		let raw = crate::sys::hook_new(hook_name.clone().into(), target_fn, replacement_fn);

		Self {
			hook_name,
			raw,
			enabled: true,
		}
	}

	/// Create a new function hook from a signature.
	///
	/// # Examples
	///
	/// ```
	/// extern "C" fn replacement_fn() {
	///    println!("Hello, world!");
	/// }
	///
	/// unsafe fn enable() {
	///   let hook = Hook::from_signature("my_hook", "48 8B 05 ? ? ? ? 48 8B 0C C8", replacement_fn as _);
	/// }
	#[allow(unused)]
	pub unsafe fn from_signature(
		hook_name: &str,
		signature: &str,
		replacement_fn: *mut c_void,
	) -> Option<Self> {
		let hook_name = hook_name.to_string();
		let result = crate::sys::hook_from_signature(
			hook_name.clone().into(),
			signature.into(),
			replacement_fn,
		);

		result.map(|raw| Self {
			hook_name,
			raw,
			enabled: true,
		})
	}
}

pub trait Hookable {
	/// Check if the hook is currently applied.
	unsafe fn is_applied(&self) -> bool;

	/// Apply the hook.
	///
	/// # Errors
	///
	/// Returns an error if the hook is already applied or if the hook failed to apply.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary and writes directly to memory.
	unsafe fn apply(&mut self) -> anyhow::Result<()>;

	/// Revert the hook.
	///
	/// # Errors
	///
	/// Returns an error if the hook is not applied or if the hook failed to revert.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary and writes directly to memory.
	unsafe fn revert(&mut self) -> anyhow::Result<()>;

	/// Check if the hook should be enabled.
	///
	/// If the hook is not enabled, the hook will not be applied when the plugin is enabled.
	fn is_enabled(&self) -> bool;

	/// Set whether the hook should be enabled.
	///
	/// If the hook is not enabled, the hook will not be applied when the plugin is enabled.
	fn set_enabled(&mut self, enabled: bool);

	/// Get the name of the hook.
	///
	/// This is the name used to identify the hook in the configuration file.
	fn get_name(&self) -> &str;

	/// Check if the hook should be enabled based on the configuration (id: `hook::hook_name`).
	///
	/// If the configuration value is not found, the default value is used.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	unsafe fn is_config_enabled(&self, plugin_id: &str) -> bool;
}

impl Hookable for Hook {
	unsafe fn is_applied(&self) -> bool {
		crate::sys::hook_is_applied(&self.raw)
	}

	unsafe fn apply(&mut self) -> anyhow::Result<()> {
		if self.is_applied() {
			Err(anyhow::anyhow!("Hook is already applied."))
		} else if crate::sys::hook_apply(&mut self.raw) {
			Ok(())
		} else {
			Err(anyhow::anyhow!("Failed to apply hook."))
		}
	}

	unsafe fn revert(&mut self) -> anyhow::Result<()> {
		if !self.is_applied() {
			Err(anyhow::anyhow!("Hook is not applied."))
		} else if crate::sys::hook_revert(&mut self.raw) {
			Ok(())
		} else {
			Err(anyhow::anyhow!("Failed to revert hook."))
		}
	}

	fn is_enabled(&self) -> bool {
		self.enabled
	}

	fn set_enabled(&mut self, enabled: bool) {
		self.enabled = enabled;
	}

	fn get_name(&self) -> &str {
		&self.hook_name
	}

	unsafe fn is_config_enabled(&self, plugin_id: &str) -> bool {
		let result = crate::sys::get_setting_bool(
			plugin_id.into(),
			format!("hook::{}", self.get_name()).into(),
		);

		match result.found {
			true => result.value,
			false => self.is_enabled(),
		}
	}
}
