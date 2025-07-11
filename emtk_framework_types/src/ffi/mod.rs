use safer_ffi::prelude::*;

#[derive_ReprC]
#[repr(C)]
pub struct GetSettingReturnValue<T> {
	pub value: T,
	pub found: bool,
}
