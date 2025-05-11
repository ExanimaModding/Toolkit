use std::{
	collections::HashMap,
	fmt::{Display, Formatter},
	path::PathBuf,
};

use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::Instance;

pub mod prelude {
	pub use crate::plugin::{self, Plugin};
}

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
/// use emcore::prelude::*;
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
/// use emcore::prelude::*;
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
	#[instrument(level = "trace")]
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
	#[instrument(level = "trace")]
	pub fn plugin_dir(&self) -> PathBuf {
		PathBuf::from(Instance::MODS_DIR).join(self.to_string())
	}

	/// Helper that returns a path to this plugin's assets directory.
	#[instrument(level = "trace")]
	pub fn assets_dir(&self) -> PathBuf {
		self.plugin_dir().join(Instance::ASSETS_DIR)
	}

	/// Helper that returns a path to this plugin's game assets directory.
	#[instrument(level = "trace")]
	pub fn packages_dir(&self) -> PathBuf {
		self.assets_dir().join(Instance::PACKAGES_DIR)
	}
}

impl TryFrom<&str> for Id {
	type Error = Error;

	#[instrument(level = "trace")]
	fn try_from(value: &str) -> Result<Self, Self::Error> {
		if !Id::is_valid(value) {
			return Err(Error::InvalidId(value.into()));
		}

		Ok(Self(value.to_string().to_lowercase()))
	}
}

impl From<Id> for String {
	#[instrument(level = "trace")]
	fn from(value: Id) -> Self {
		value.0
	}
}

impl Display for Id {
	#[instrument(level = "trace", skip(f))]
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
