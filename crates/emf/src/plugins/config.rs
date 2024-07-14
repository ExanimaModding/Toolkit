use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PluginInfo {
	pub config: PluginConfig,
	pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
	/// The plugins that are available.
	pub plugin: PluginConfigPlugin,

	/// The settings for the plugin.
	#[serde(rename = "setting")]
	pub settings: Vec<PluginConfigSetting>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfigPlugin {
	/// The unique ID of a plugin
	///
	/// e.g. `"com.yourusername.pluginname"`
	pub id: String,

	/// The name of the plugin
	///
	/// e.g. `"Plugin Name"`
	pub name: String,

	/// The version of the plugin.
	///
	/// e.g. `"1.0.0"`
	pub version: String,

	/// The supported game versions.
	pub supported_versions: Vec<String>,

	/// The URL to the plugin source code.
	///
	/// e.g. `"https://codeberg.org/ExanimaModding/Toolkit"``
	pub url: String,

	/// The Author of the plugin.
	pub author: PluginConfigAuthor,

	/// The executable name.
	///
	/// This is optional, as some mods only provide asset changes.
	///
	/// e.g. `"com.yourusername.pluginname.dll"`
	pub executable: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfigAuthor {
	/// The name of the author.
	///
	/// e.g. `"Megumin"`
	pub name: String,

	/// Free-form contact for the author (optional).
	///
	/// e.g. `"Discord: @Megumin"`
	///
	/// e.g. `"Email: your@email.com"`
	pub contact: Option<String>,

	/// The URL of the author (optional).
	///
	/// e.g. `"https://megu.dev"`
	pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfigSetting {
	/// The name of the setting.
	///
	/// e.g. `"Enable Plugin"`
	pub name: String,

	/// The description of the setting.
	///
	/// e.g. `"Enable or disable the plugin."`
	pub description: String,

	/// The default value of the setting.
	///
	/// e.g. `true`
	pub default: PluginConfigSettingValue,

	/// The value of the setting.
	///
	/// e.g. `true`
	///
	/// e.g. `"Hello, World!"`
	///
	/// e.g. `42`
	pub value: Option<PluginConfigSettingValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum PluginConfigSettingValue {
	Boolean(bool),
	String(String),
	Number(f64),
}
