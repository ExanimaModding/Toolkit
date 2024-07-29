/// A byte patch that modifies process memory directly.
pub struct Patch {
	/// The identifier of the Patch.
	name: String,

	/// The address in memory to patch.
	address: *const u8,

	/// The bytes to write to memory.
	patch_bytes: Vec<u8>,
	// TODO: Should this be removed?
	#[allow(dead_code)]
	/// The original bytes at the address.
	original_bytes: safer_ffi::Vec<u8>,

	/// The raw patch object.
	raw: safer_ffi::boxed::Box<crate::sys::PatchRaw>,

	/// Whether the patch should be used when the plugin is enabled.
	enabled: bool,
}

impl Patch {
	/// Create a new patch.
	///
	/// # Examples
	///
	/// ```
	/// extern "C" fn enable() {
	///     let ptr = 0xDEADBEEF as *const u8;
	///     let patch = vec![0x90, 0x90, 0x90, 0x90];
	///     let patch = Patch::new("my_patch", ptr, patch);
	/// }
	pub unsafe fn new(name: &str, address: *const u8, patch_bytes: Vec<u8>) -> Self {
		let name = name.to_string();

		let original_bytes = crate::sys::read_bytes(address as _, patch_bytes.len());
		let raw = crate::sys::patch_new(address as _, patch_bytes.clone().into());

		Self {
			name,
			address,
			patch_bytes,
			original_bytes,
			raw,
			enabled: true,
		}
	}
}

pub trait Patchable {
	/// Check if the patch is currently applied.
	unsafe fn is_applied(&self) -> bool;

	/// Apply the patch.
	///
	/// # Errors
	///
	/// Returns an error if the patch is already applied or if the patch failed to apply.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary and writes directly to memory.
	unsafe fn apply(&mut self) -> anyhow::Result<()>;

	/// Revert the patch.
	///
	/// # Errors
	///
	/// Returns an error if the patch is not applied or if the patch failed to revert.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary and writes directly to memory.
	unsafe fn revert(&mut self) -> anyhow::Result<()>;

	/// Get the bytes currently written to memory.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	unsafe fn get_current_bytes(&self) -> Vec<u8>;

	/// Check if the patch should be enabled.
	///
	/// If the patch is not enabled, the patch will not be applied when the plugin is enabled.
	fn is_enabled(&self) -> bool;

	/// Set whether the patch should be enabled.
	///
	/// If the patch is not enabled, the patch will not be applied when the plugin is enabled.
	fn set_enabled(&mut self, enabled: bool);

	/// Get the name of the patch.
	fn get_name(&self) -> &str;

	/// Check if the patch should be enabled based on the configuration (id: `patch::patch_name`).
	///
	/// If the configuration value is not found, the default value is used.
	///
	/// # Safety
	///
	/// This function is unsafe because it crosses the FFI boundary.
	unsafe fn is_config_enabled(&self, plugin_id: &str) -> bool;
}

impl Patchable for Patch {
	unsafe fn is_applied(&self) -> bool {
		crate::sys::patch_is_applied(&self.raw)
	}

	unsafe fn apply(&mut self) -> anyhow::Result<()> {
		if self.is_applied() {
			Err(anyhow::anyhow!("Patch is already applied."))
		} else if crate::sys::patch_apply(&mut self.raw) {
			Ok(())
		} else {
			Err(anyhow::anyhow!("Failed to apply patch."))
		}
	}

	unsafe fn revert(&mut self) -> anyhow::Result<()> {
		if !self.is_applied() {
			Err(anyhow::anyhow!("Patch is not applied."))
		} else if crate::sys::patch_revert(&mut self.raw) {
			Ok(())
		} else {
			Err(anyhow::anyhow!("Failed to revert patch."))
		}
	}

	unsafe fn get_current_bytes(&self) -> Vec<u8> {
		crate::sys::read_bytes(self.address as _, self.patch_bytes.len()).into()
	}

	fn is_enabled(&self) -> bool {
		self.enabled
	}

	fn set_enabled(&mut self, enabled: bool) {
		self.enabled = enabled;
	}

	fn get_name(&self) -> &str {
		self.name.as_str()
	}

	unsafe fn is_config_enabled(&self, plugin_id: &str) -> bool {
		let result = crate::sys::get_setting_bool(
			plugin_id.into(),
			format!("patch::{}", self.get_name()).into(),
		);

		match result.found {
			true => result.value,
			false => self.is_enabled(),
		}
	}
}
