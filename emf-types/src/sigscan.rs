use std::ffi::{c_char, CString};

#[repr(C)]
pub struct SigScanPayloadRaw {
	/// The pattern to scan for. Unknown bytes can be replaced with "??".
	/// e.g. "30 F6 8B 05 ?? ?? ?? ?? 89"
	pub pattern: *mut c_char,
}

impl SigScanPayloadRaw {
	pub fn get_pattern(&self) -> CString {
		unsafe { CString::from_raw(self.pattern) }
	}

	pub fn serialize(&self) -> SigScanPayload {
		SigScanPayload {
			pattern: self.get_pattern(),
		}
	}
}

#[derive(Debug)]
pub struct SigScanPayload {
	/// The pattern to scan for. Unknown bytes can be replaced with "??".
	/// e.g. "30 F6 8B 05 ?? ?? ?? ?? 89"
	pub pattern: CString,
}

impl SigScanPayload {
	pub fn deserialize(&self) -> SigScanPayloadRaw {
		SigScanPayloadRaw {
			pattern: CString::into_raw(self.pattern.clone()),
		}
	}
}
