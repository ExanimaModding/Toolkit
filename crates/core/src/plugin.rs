use std::{
	env,
	ffi::{self, CString},
	fmt::{Display, Formatter},
	mem,
	path::PathBuf,
	sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use tracing::instrument;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

use super::{Instance, Result};

pub mod prelude {
	pub use crate::plugin;
}

/// The name of the file used as the entry point to lua scripting for the
/// plugin.
pub static LUA: &str = "plugin.lua";

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0} is not valid reverse domain name notation")]
	InvalidId(String),
	#[error("Dll {0} not found")]
	DllNotFound(String),
	#[error("Dll {0} does not export {1}")]
	DllExportNotFound(String, String),
	#[error("failed to get plugin list")]
	PluginListError,
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
///     Err(_invalid_id) => {
///         // handle invalid id here
///         // _invalid_id would be "com.example.my-mod" in this case
///         return;
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
}

impl TryFrom<&str> for Id {
	type Error = Error;

	fn try_from(value: &str) -> std::result::Result<Self, Error> {
		if !Id::is_valid(value) {
			return Err(Error::InvalidId(value.into()));
		}

		Ok(Self(value.to_string().to_lowercase()))
	}
}

impl TryFrom<String> for Id {
	type Error = Error;

	fn try_from(value: String) -> std::result::Result<Self, Error> {
		Self::try_from(value.as_str())
	}
}

impl Display for Id {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

type EmfGetPLuginList = unsafe extern "C" fn(mods_dir: *const ffi::c_char) -> *mut ffi::c_char;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
	/// The display name of the plugin associated to this manifest. This field is
	/// not used to identify the plugin.
	pub name: String,

	/// The version of the plugin associated to this manifest. Semantic versioning
	/// will be best practice in the format major, minor, patch, a.k.a. 0.1.0
	pub version: String,

	/// The creator of the plugin associated to this manifest.
	pub author: String,

	/// The list of plugin Ids that the plugin associated to this manifest is
	/// required by in order to function properly.
	pub dependencies: Vec<Id>,

	/// The list of plugin Ids that the plugin associated to this manifest is
	/// incompatible with.
	pub conflicts: Vec<Id>,
}

impl Manifest {
	/// The name of the key responsible for storing the value to the author of the
	/// plugin. This key is defined in the mod's [`emcore::plugin::LUA`] file.
	pub const AUTHOR: &str = "author";

	/// The name of the key responsible for storing a list of plugin ids that are
	/// incompatible with this plugin. This key is defined in the mod's
	/// [`emcore::plugin::LUA`] file.
	pub const CONFLICTS: &str = "conflicts";

	/// The name of the key responsible for storing a list of plugin ids that are
	/// required by this plugin in order to function properly. This key is defined in
	/// the mod's [`emcore::plugin::LUA`] file.
	pub const DEPENDENCIES: &str = "dependencies";

	/// The name of the key responsible for storing the value to the display name of
	/// the plugin. This key is defined in the mod's [`emcore::plugin::LUA`] file.
	pub const NAME: &str = "name";

	/// The name of the key responsible for storing the value to the version of the
	/// plugin. This key is defined in the mod's [`emcore::plugin::LUA`] file.
	pub const VERSION: &str = "version";

	// PERF: emf being loaded into memory may be slowing initialization
	#[instrument(level = "trace")]
	pub fn discover_mods(mods_dir: &PathBuf) -> Result<Vec<(Id, Manifest)>> {
		static GET_PLUGIN_LIST: OnceLock<Result<EmfGetPLuginList>> = OnceLock::new();

		GET_PLUGIN_LIST.get_or_init(|| {
			// We set this so that the DllMain doesn't try to hook when running from EMTK.
			unsafe { env::set_var("RUST_LIBLOADING", "1") };

			#[cfg(debug_assertions)]
			let h_module = unsafe { LoadLibraryA(c"deps/emf.dll".as_ptr() as _) };
			#[cfg(not(debug_assertions))]
			let h_module = unsafe { LoadLibraryA(c"emf.dll".as_ptr() as _) };

			// Remove it after DllMain is called, to make sure EMTK can still inject EMF properly later.
			unsafe { env::remove_var("RUST_LIBLOADING") };

			if h_module.is_null() {
				return Err(crate::Error::new(
					Error::DllNotFound("emf.dll".to_string()),
					"failed to get Dll",
				));
			}

			let get_plugin_list =
				unsafe { GetProcAddress(h_module, c"EmfGetPluginList".as_ptr() as _) };

			let Some(get_plugin_list) = get_plugin_list else {
				return Err(crate::Error::new(
					Error::DllExportNotFound("emf.dll".to_string(), "EmfGetPluginList".to_string()),
					"failed to get plugin list",
				));
			};

			let get_plugin_list: EmfGetPLuginList = unsafe { mem::transmute(get_plugin_list) };

			Ok(get_plugin_list)
		});

		let Ok(get_plugin_list) = GET_PLUGIN_LIST.get().unwrap() else {
			return Err(crate::Error::new(
				Error::PluginListError,
				"failed to get plugin list",
			));
		};

		let mods_dir = CString::new(mods_dir.display().to_string()).map_err(crate::Error::msg(
			"failed to create new C string for mods directory",
		))?;
		let mods_dir = mods_dir.as_ptr();

		let response = unsafe { get_plugin_list(mods_dir) };

		if response.is_null() {
			return Err(crate::Error::new(
				Error::PluginListError,
				"failed to get plugin list",
			));
		}

		// Take ownership of the memory first with Box, then convert to CString
		let boxed_response = unsafe { Box::from_raw(response as *mut ffi::c_char) };
		let response = unsafe { ffi::CString::from_raw(Box::into_raw(boxed_response)) };

		let response = response.to_str().map_err(crate::Error::msg(
			"failed to convert plugin list into string",
		))?;

		ron::from_str(response).map_err(crate::Error::msg("failed to get plugin list"))
	}
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
