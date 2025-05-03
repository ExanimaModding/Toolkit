use std::{
	env,
	ffi::{self, CStr, CString},
	fmt::{Display, Formatter},
	io,
	path::PathBuf,
	sync::OnceLock,
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
	#[error("failed to decode manifest property '{0}'")]
	DecodeError(&'static str),
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
#[repr(C)]
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

impl TryFrom<String> for Id {
	type Error = Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		Self::try_from(value.as_str())
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

#[derive(Debug)]
#[repr(C)]
pub struct Manifest {
	pub id: Id,
	pub path: String,
	pub name: String,
	pub version: String,
	pub author: String,
	pub dependencies: Vec<Id>,
	pub conflicts: Vec<Id>,
}

#[derive(Debug)]
#[repr(C)]
pub struct SizedString {
	pub ptr: *mut ffi::c_char,
	pub len: usize,
}

impl TryFrom<SizedString> for String {
	type Error = Error;

	fn try_from(value: SizedString) -> Result<Self, Self::Error> {
		if value.ptr.is_null() {
			return Err(Error::DecodeError("string"));
		}

		let bytes = unsafe { std::slice::from_raw_parts(value.ptr as _, value.len) };

		let Ok(string) = std::str::from_utf8(bytes) else {
			return Err(Error::DecodeError("string"));
		};

		Ok(string.to_string())
	}
}

impl TryFrom<String> for SizedString {
	type Error = Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let bytes = value.into_bytes();

		let (ptr, len, _) = bytes.into_raw_parts();

		Ok(Self { ptr: ptr as _, len })
	}
}

#[derive(Debug)]
#[repr(C)]
pub struct CManifest {
	pub id: SizedString,
	pub path: SizedString,
	pub name: SizedString,
	pub version: SizedString,
	pub author: SizedString,
	pub dependencies: *mut SizedString,
	pub dependencies_count: usize,
	pub dependencies_capacity: usize,
	pub conflicts: *mut SizedString,
	pub conflicts_count: usize,
	pub conflicts_capacity: usize,
}

impl TryFrom<CManifest> for Manifest {
	type Error = Error;

	fn try_from(value: CManifest) -> Result<Self, Self::Error> {
		let Ok(id) = String::try_from(value.id) else {
			return Err(Error::DecodeError("id"));
		};
		let Ok(id) = Id::try_from(id.clone()) else {
			return Err(Error::InvalidId(id));
		};
		let Ok(path) = String::try_from(value.path) else {
			return Err(Error::DecodeError("path"));
		};
		let Ok(name) = String::try_from(value.name) else {
			return Err(Error::DecodeError("name"));
		};
		let Ok(version) = String::try_from(value.version) else {
			return Err(Error::DecodeError("version"));
		};
		let Ok(author) = String::try_from(value.author) else {
			return Err(Error::DecodeError("author"));
		};

		let dependencies: Vec<Id> = if value.dependencies.is_null() {
			Vec::new()
		} else {
			unsafe {
				Vec::from_raw_parts(
					value.dependencies,
					value.dependencies_count,
					value.dependencies_capacity,
				)
				.into_iter()
				.filter_map(|ptr| String::try_from(ptr).ok())
				.filter_map(|id_str| Id::try_from(id_str).ok())
				.collect()
			}
		};
		println!("meow");

		let conflicts: Vec<Id> = if value.conflicts.is_null() {
			Vec::new()
		} else {
			unsafe {
				Vec::from_raw_parts(
					value.conflicts,
					value.conflicts_count,
					value.conflicts_capacity,
				)
				.into_iter()
				.map(|ptr| String::try_from(ptr))
				.flatten()
				.filter_map(|id_str| Id::try_from(id_str).ok())
				.collect()
			}
		};

		println!("Done without crashing 1");

		Ok(Self {
			id,
			path,
			name,
			version,
			author,
			dependencies,
			conflicts,
		})
	}
}

impl TryFrom<Manifest> for CManifest {
	type Error = Error;

	fn try_from(value: Manifest) -> Result<Self, Self::Error> {
		let Ok(id) = SizedString::try_from(value.id.to_string()) else {
			return Err(Error::DecodeError("id"));
		};
		let Ok(path) = SizedString::try_from(value.path) else {
			return Err(Error::DecodeError("path"));
		};
		let Ok(name) = SizedString::try_from(value.name) else {
			return Err(Error::DecodeError("name"));
		};
		let Ok(version) = SizedString::try_from(value.version) else {
			return Err(Error::DecodeError("version"));
		};
		let Ok(author) = SizedString::try_from(value.author) else {
			return Err(Error::DecodeError("author"));
		};

		let dependencies: Vec<SizedString> = value
			.dependencies
			.into_iter()
			.map(|id| SizedString::try_from(id.to_string()).unwrap())
			.collect();

		let (dependencies, dependencies_len, dependencies_cap) = if dependencies.is_empty() {
			(std::ptr::null_mut(), 0, 0)
		} else {
			dependencies.into_raw_parts()
		};

		let conflicts: Vec<SizedString> = value
			.conflicts
			.into_iter()
			.map(|id| SizedString::try_from(id.to_string()).unwrap())
			.collect();

		let (conflicts, conflicts_len, conflicts_cap) = if conflicts.is_empty() {
			(std::ptr::null_mut(), 0, 0)
		} else {
			conflicts.into_raw_parts()
		};

		Ok(Self {
			id,
			path,
			name,
			version,
			author,
			dependencies,
			dependencies_count: dependencies_len,
			dependencies_capacity: dependencies_cap,
			conflicts,
			conflicts_count: conflicts_len,
			conflicts_capacity: conflicts_cap,
		})
	}
}

#[repr(C)]
pub struct GetPluginListResponse {
	pub plugins: *mut CManifest,
	pub count: usize,
}

// #[link(name = "emf.dll")]
// unsafe extern "C" {
// 	fn EMF_GetPluginList(mods_dir: *const ffi::c_char) -> *mut GetPluginListResponse;
// }

#[allow(non_camel_case_types)]
type EMF_GetPluginList =
	unsafe extern "C" fn(mods_dir: *const ffi::c_char) -> *mut GetPluginListResponse;

impl Manifest {
	pub fn discover_mods(mods_dir: &PathBuf) -> Result<Vec<Manifest>, io::Error> {
		static EMF_DLL: OnceLock<Option<libloading::Library>> = OnceLock::new();
		#[cfg(debug_assertions)]
		EMF_DLL.get_or_init(|| unsafe {
			env::set_var("RUST_LIBLOADING", "1");
			let lib = libloading::Library::new("deps/emf.dll").ok();
			env::remove_var("RUST_LIBLOADING");
			lib
		});
		#[cfg(not(debug_assertions))]
		EMF_DLL.get_or_init(|| unsafe {
			env::set_var("RUST_LIBLOADING", "1");
			let lib = libloading::Library::new("emf.dll").ok();
			env::remove_var("RUST_LIBLOADING");
			lib
		});

		let Some(Some(lib)) = EMF_DLL.get() else {
			return Err(io::Error::new(io::ErrorKind::Other, "emf.dll not found!"));
		};

		let get_plugin_list: libloading::Symbol<EMF_GetPluginList> =
			unsafe { lib.get(b"EMF_GetPluginList\0").unwrap() };

		let Some(mods_dir) = mods_dir.to_str() else {
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				"mods_dir is not a valid string!",
			));
		};

		let mods_dir = CString::new(mods_dir).map_err(io::Error::from)?;
		let mods_dir = mods_dir.as_ptr();

		let response = unsafe { get_plugin_list(mods_dir) };

		if response.is_null() {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"EMF_GetPluginList returned a null pointer!",
			));
		}

		let response = unsafe { Box::from_raw(response) };

		let plugins =
			unsafe { Vec::from_raw_parts(response.plugins, response.count, response.count) };

		dbg!(&plugins);

		let plugins: Vec<Result<Manifest, Error>> =
			plugins.into_iter().map(Manifest::try_from).collect();

		dbg!("done 2");

		let plugins = plugins.into_iter().flatten().collect();

		Ok(plugins)
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
