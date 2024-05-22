use std::ffi::{c_char, CString};

#[repr(C)]
#[derive(Debug)]
pub struct PluginInfo {
	/// The unique identifier of the plugin.
	/// e.g. "com.yourusername.pluginname"
	pub plugin_id: CString,
}

#[repr(C)]
#[derive(Debug)]
pub struct PluginMessageRaw {
	/// The sender of the message (e.g. "com.yourusername.pluginname").
	pub from: *mut c_char,
	/// The recipient of the message (e.g. "com.someusername.pluginname").
	pub to: *mut c_char,
	/// The message to send, as a byte array.
	pub message: *const u8,
	/// The length of the message.
	pub message_len: usize,
}

impl PluginMessageRaw {
	pub fn serialize(&self) -> PluginMessage {
		unsafe {
			PluginMessage {
				from: CString::from_raw(self.from),
				to: CString::from_raw(self.to),
				message: std::slice::from_raw_parts(self.message, self.message_len).to_vec(),
			}
		}
	}
}

#[derive(Debug)]
pub struct PluginMessage {
	/// The sender of the message (e.g. "com.yourusername.pluginname").
	pub from: CString,
	/// The recipient of the message (e.g. "com.someusername.pluginname").
	pub to: CString,
	/// The message to send, as a Vec<u8>.
	pub message: Vec<u8>,
}

impl PluginMessage {
	pub fn deserialize(&self) -> PluginMessageRaw {
		let (ptr, len, _) = Vec::into_raw_parts(self.message.clone());
		PluginMessageRaw {
			from: CString::into_raw(self.from.clone()),
			to: CString::into_raw(self.to.clone()),
			message: ptr,
			message_len: len,
		}
	}
}

pub type GetPluginInfoFn = extern "C" fn() -> *const PluginInfo;
pub type PluginInitFn = extern "C" fn() -> bool;

/// When a plugin receives a message from another plugin.
pub type OnMessageFn = extern "C" fn(*const PluginMessageRaw);
// Send a message from a plugin to another plugin.
pub type SendMessageFn = extern "C" fn(*const PluginMessageRaw);
