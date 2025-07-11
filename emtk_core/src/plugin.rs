use std::{
	collections::HashMap,
	fmt::{Display, Formatter},
	path::PathBuf,
};

use serde::{Deserialize, Serialize};

use super::Instance;

pub mod prelude {
	pub use crate::plugin::{self, Plugin};
}

/// The name of file used to provide more information about the plugin in
/// markdown.
pub const README: &str = "README.md";

/// The name of the file used to provide changelogs about the plugin in
/// markdown.
pub const CHANGELOG: &str = "CHANGELOG.md";

/// The name of the file used to provide licensing information of the plugin.
pub const LICENSE: &str = "LICENSE";

#[derive(PartialEq, Eq, Hash, Debug, thiserror::Error)]
pub enum Error {
	/// Contains the Id that caused the error
	#[error("id, {0}, must be in reverse domain name notation")]
	InvalidId(String),
}

/// An ID represented in [reverse domain name notation] and should be stored
/// as such.
///
/// [reverse domain name notation]: https://en.wikipedia.org/wiki/Reverse_domain_name_notation
///
/// `Id` must be alphanumeric with the exceptions for '-' and '.' as separators.
/// Be aware this is case-insensitive for compatibility reasons with Windows.
///
/// # Examples
///
/// `Id` is created using a string:
///
/// ```rust
/// use emtk_core::prelude::*;
///
/// let my_plugin_id = match plugin::Id::try_from("com.example.my-mod") {
///     Ok(id) => id,
///     Err(e) => match e {
///         plugin::Error::InvalidId(_invalid_id) => {
///             // handle invalid id here
///             // _invalid_id would be "com.example.my-mod" in this case
///             return;
///         },
///         _ => return,
///     },
/// };
/// ```
///
/// If you have two similar `Id`s with differing capitalization, they are equal:
///
/// ```rust
/// use emtk_core::prelude::*;
///
/// // Keep in mind these are equal
/// assert_eq!(
///     plugin::Id::try_from("com.example.MyMod").unwrap(),
///     plugin::Id::try_from("com.example.mymod").unwrap()
/// )
/// ```
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Id(String);

impl Id {
	/// The following conditions will return false:
	///
	/// - Is empty
	/// - Starts or ends with '-' or '.'
	/// - Not alphanumeric (exceptions: '-', '.')
	pub fn is_valid(id: &str) -> bool {
		if id.is_empty()
			|| id.starts_with(['-', '.'])
			|| id.ends_with(['-', '.'])
			|| !id
				.chars()
				.all(|chr| chr.is_alphanumeric() || chr == '-' || chr == '.')
		{
			return false;
		};

		true
	}

	/// Helper that returns a path to this plugin's directory
	pub fn plugin_dir(&self) -> PathBuf {
		PathBuf::from(Instance::MODS_DIR).join(self.to_string())
	}

	/// Helper that returns a path to this plugin's assets directory.
	pub fn assets_dir(&self) -> PathBuf {
		self.plugin_dir().join(Instance::ASSETS_DIR)
	}

	/// Helper that returns a path to this plugin's game assets directory.
	pub fn packages_dir(&self) -> PathBuf {
		self.assets_dir().join(Instance::PACKAGES_DIR)
	}

	/// Helper that returns a path to this plugin's [`README`] file.
	pub fn readme_file(&self) -> PathBuf {
		self.plugin_dir().join(README)
	}

	/// Helper that returns a path to this plugin's [`CHANGELOG`] file.
	pub fn changelog_file(&self) -> PathBuf {
		self.plugin_dir().join(CHANGELOG)
	}

	/// Helper that returns a path to this plugin's [`LICENSE`] file.
	pub fn license_file(&self) -> PathBuf {
		self.plugin_dir().join(LICENSE)
	}

	/// Helper that returns a path to this plugin's [`SETTINGS`] file.
	pub fn settings_file(&self) -> PathBuf {
		self.plugin_dir().join(Settings::TOML)
	}
}

impl TryFrom<&str> for Id {
	type Error = Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		if !Id::is_valid(value) {
			return Err(Error::InvalidId(value.into()));
		}

		Ok(Self(value.to_string().to_lowercase()))
	}
}

impl From<Id> for String {
	fn from(value: Id) -> Self {
		value.0
	}
}

impl Display for Id {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Plugin {
	/// The display name of the plugin
	pub name: String,
	/// The version of the plugin. Semantic versioning will be best practice in the
	/// format major, minor, patch, a.k.a. v0.1.0
	pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Dependency {
	Version(String),
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct Dependency {
// 	version: String,
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct Conflicts {
	version: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
	pub plugin: Plugin,
	#[serde(default)]
	pub conflicts: Option<HashMap<Id, Conflicts>>,
	#[serde(default)]
	pub dependencies: Option<HashMap<Id, Dependency>>,
}

impl Manifest {
	/// The name of the file responsible for storing information about the plugin
	/// such as display name, version, dependencies, etc.
	pub const TOML: &str = "manifest.toml";
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Widget {
	Checkbox(Checkbox),
	Dropdown(Dropdown),
	Radio(Radio),
	Slider(Slider),
	TextInput(TextInput),
}

impl From<Checkbox> for Widget {
	fn from(value: Checkbox) -> Self {
		Widget::Checkbox(value)
	}
}

impl From<Dropdown> for Widget {
	fn from(value: Dropdown) -> Self {
		Widget::Dropdown(value)
	}
}

impl From<Radio> for Widget {
	fn from(value: Radio) -> Self {
		Widget::Radio(value)
	}
}

impl From<Slider> for Widget {
	fn from(value: Slider) -> Self {
		Widget::Slider(value)
	}
}

impl From<TextInput> for Widget {
	fn from(value: TextInput) -> Self {
		Widget::TextInput(value)
	}
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Settings {
	#[serde(rename = "widget")]
	#[serde(default)]
	pub widgets: Vec<Widget>,
}

impl Settings {
	/// The name of the file used to customize plugin behavior on load.
	pub const TOML: &str = "settings.toml";
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Checkbox {
	pub about: Option<String>,
	pub label: String,
	pub value: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DropdownItem {
	pub about: Option<String>,
	pub label: String,
	// pub value: bool,
	// pub value: toml::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dropdown {
	pub about: Option<String>,
	pub label: String,
	pub value: i64,
	pub options: Vec<DropdownItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RadioButton {
	pub about: Option<String>,
	pub label: String,
	// pub value: bool,
	// pub value: toml::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Radio {
	pub about: Option<String>,
	pub label: String,
	pub value: i64,
	pub options: Vec<RadioButton>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Slider {
	pub about: Option<String>,
	pub label: String,
	pub value: f64,
	pub range: [f64; 2],
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextInput {
	pub about: Option<String>,
	pub label: String,
	pub value: String,
}

#[cfg(test)]
mod tests {
	use pretty_assertions::{assert_eq, assert_ne};

	use crate::prelude::*;

	#[test]
	fn id_is_equal() {
		assert_eq!(
			plugin::Id::try_from("com.example.my-mod").unwrap(),
			plugin::Id::try_from("com.example.my-mod").unwrap()
		);
		assert_eq!(
			plugin::Id::try_from("com.example.my-mod").unwrap(),
			plugin::Id::try_from("com.example.My-Mod").unwrap() // case-insensitive
		);
	}

	#[test]
	fn id_is_not_equal() {
		assert_ne!(
			plugin::Id::try_from("com.example.my-mod").unwrap(),
			plugin::Id::try_from("com.example.my-other-mod").unwrap()
		);
	}

	#[test]
	fn id_is_valid() {
		assert!(plugin::Id::is_valid("com.example.my-mod"));
		assert!(plugin::Id::is_valid("com.example.my-mod-2"));
	}

	#[test]
	fn id_is_not_valid() {
		assert!(!plugin::Id::is_valid("com.example.my mod")); // whitespace
		assert!(!plugin::Id::is_valid("com.example.my-mod ")); // trailing whitespace
		assert!(!plugin::Id::is_valid(""));
		assert!(!plugin::Id::is_valid("com.example.\nmy-mod")); // newline
		assert!(!plugin::Id::is_valid("\n"));
		assert!(!plugin::Id::is_valid("com.example.my_mod")); // underscore
		assert!(!plugin::Id::is_valid("com.example.my#mod")); // #
		assert!(!plugin::Id::is_valid("com.example.my-mod.")); // ends with .
		assert!(!plugin::Id::is_valid("."));
		assert!(!plugin::Id::is_valid("-com.example.my-mod")); // begins with -
		assert!(!plugin::Id::is_valid("-"));
	}
}
