use std::ffi::{c_char, c_void, CString};

#[derive(Debug)]
#[repr(C)]
pub struct ReadMemoryPayloadRaw {
	/// The address of the memory to read.
	pub target_address: *mut c_void,
	/// The number of bytes to read.
	pub read_length: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct PatchMemoryPayloadRaw {
	/// The name of the patch. You can use this to un-patch the memory later.
	pub patch_name: *mut c_char,
	/// The address of the memory to patch.
	pub target_address: *mut c_void,
	/// The bytes to write to the memory.
	pub replacement_bytes: *mut u8,
	/// The number of bytes to write to the memory.
	pub replacement_length: usize,
}

impl PatchMemoryPayloadRaw {
	pub fn get_patch_name(&self) -> CString {
		unsafe { CString::from_raw(self.patch_name) }
	}
	pub fn serialize(&self) -> PatchMemoryPayload {
		PatchMemoryPayload {
			patch_name: CString::new(self.get_patch_name()).unwrap(),
			target_address: self.target_address,
			replacement_bytes: unsafe {
				Vec::from_raw_parts(
					self.replacement_bytes,
					self.replacement_length,
					self.replacement_length,
				)
			},
		}
	}
}

#[derive(Debug)]
pub struct PatchMemoryPayload {
	/// The name of the patch. You can use this to un-patch the memory later.
	pub patch_name: CString,
	/// The address of the memory to patch.
	pub target_address: *mut c_void,
	/// The bytes to write to the memory.
	pub replacement_bytes: Vec<u8>,
}

impl PatchMemoryPayload {
	pub fn deserialize(&self) -> PatchMemoryPayloadRaw {
		let (ptr, len, _) = Vec::into_raw_parts(self.replacement_bytes.clone());

		PatchMemoryPayloadRaw {
			patch_name: CString::into_raw(self.patch_name.clone()),
			target_address: self.target_address,
			replacement_bytes: ptr,
			replacement_length: len,
		}
	}
}
