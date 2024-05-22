use emf_types::plugin::{GetPluginInfoFn, OnMessageFn, PluginInitFn};

#[derive(Debug)]

pub struct PluginState {
	pub loaded: bool,
	pub enabled: bool,
	pub plugin_id: String,
	pub lib: Option<libloading::Library>,

	pub init: Option<PluginInitFn>,
	pub get_info: Option<GetPluginInfoFn>,
	pub send_message: Option<OnMessageFn>,
}
