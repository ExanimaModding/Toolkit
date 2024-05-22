use std::ffi::{c_char, c_void, CString};

// TODO: Eviscerate this when #[ffi_export] is implemented.

#[derive(Debug)]
#[repr(C)]
pub struct HookFunctionPayloadRaw {
	/// The name of the hook. You can use this to un-hook the function later.
	pub hook_name: *mut c_char,
	/// The function to override.
	/// The value you enter here will become the *trampoline* pointer, which you can use to call the original function.
	pub target_function_ptr: *mut *mut c_void,
	/// The function to replace the target function with.
	pub replacement_function_ptr: *mut c_void,
}

impl HookFunctionPayloadRaw {
	pub fn get_hook_name(&self) -> CString {
		unsafe { CString::from_raw(self.hook_name) }
	}
	pub fn serialize(&self) -> HookFunctionPayload {
		HookFunctionPayload {
			hook_name: CString::new(self.get_hook_name()).unwrap(),
			target_function_ptr: self.target_function_ptr,
			replacement_function_ptr: self.replacement_function_ptr,
		}
	}
}

#[derive(Debug)]
pub struct HookFunctionPayload {
	/// The name of the hook. You can use this to un-hook the function later.
	pub hook_name: CString,
	/// The function to override.
	/// The value you enter here will become the *trampoline* pointer, which you can use to call the original function.
	pub target_function_ptr: *mut *mut c_void,
	/// The function to replace the target function with.
	pub replacement_function_ptr: *mut c_void,
}

impl HookFunctionPayload {
	pub fn deserialize(&self) -> HookFunctionPayloadRaw {
		HookFunctionPayloadRaw {
			hook_name: CString::into_raw(self.hook_name.clone()),
			target_function_ptr: self.target_function_ptr,
			replacement_function_ptr: self.replacement_function_ptr,
		}
	}
}
