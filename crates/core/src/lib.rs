#![doc = include_str!("../README.md")]
#![feature(let_chains)]
#![feature(error_generic_member_access)]
#![deny(clippy::panic)]
#![deny(clippy::expect_used)]
#![deny(clippy::unwrap_used)]

pub mod cache;
pub mod instance;
pub mod plugin;
pub mod profile;

use std::{
	backtrace::Backtrace,
	borrow::Cow,
	ffi, fmt,
	path::{Path, PathBuf},
	str,
	time::SystemTimeError,
};

use tokio::{fs, io};

pub use instance::{Instance, InstanceHistory};
pub use plugin::Plugin;
pub use profile::Profile;
use tracing::instrument;

pub mod prelude {
	pub use crate::{instance::prelude::*, plugin::prelude::*, profile::prelude::*};
}

pub type Result<T> = std::result::Result<T, Error>;

/// Errors related to the core crate.
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub enum CoreError {
	Plugin {
		#[from]
		source: plugin::Error,
		backtrace: Backtrace,
	},
}

/// Errors related to serializing and deserializing from/to files.
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub enum SerdeError {
	Ron {
		#[from]
		source: ron::Error,
		backtrace: Backtrace,
	},
	#[error(transparent)]
	Toml(#[from] TomlError),
}

#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub enum TomlError {
	Deserialize {
		#[from]
		source: toml::de::Error,
		backtrace: Backtrace,
	},
	Serialize {
		#[from]
		source: toml::ser::Error,
		backtrace: Backtrace,
	},
}

/// Errors related to the rust's standard library.
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub enum StdError {
	Io {
		#[from]
		source: std::io::Error,
		backtrace: Backtrace,
	},
	Nul {
		#[from]
		source: ffi::NulError,
		backtrace: Backtrace,
	},
	Time {
		#[from]
		source: SystemTimeError,
		backtrace: Backtrace,
	},
	Utf8 {
		#[from]
		source: str::Utf8Error,
		backtrace: Backtrace,
	},
}

#[derive(Debug, thiserror::Error)]
#[error("{message}: {kind}")]
pub struct Error {
	kind: ErrorKind,
	message: Cow<'static, str>,
}

impl Error {
	#[instrument(level = "trace")]
	pub fn new<K, M>(kind: K, message: M) -> Self
	where
		K: Into<ErrorKind> + fmt::Debug,
		M: Into<Cow<'static, str>> + fmt::Debug,
	{
		Self {
			kind: kind.into(),
			message: message.into(),
		}
	}

	#[instrument(level = "trace")]
	pub fn msg<K, M>(message: M) -> impl FnOnce(K) -> Self
	where
		K: Into<ErrorKind> + fmt::Debug,
		M: Into<Cow<'static, str>> + fmt::Debug,
	{
		move |e| Self::new(e, message)
	}
}

/// The source of truth of the types of errors that are involved in the core
/// crate.
#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub enum ErrorKind {
	/// Errors related to the core crate. If the error comes from a third party
	/// crate, a new enum variant should be created in [`ErrorKind`].
	#[error(transparent)]
	Core(#[from] CoreError),
	/// Errors related to serializing and deserializing from/to files.
	#[error(transparent)]
	Serde(#[from] SerdeError),
	/// Errors related to the rust's standard library.
	#[error(transparent)]
	Std(#[from] StdError),
	/// Convenient public interface for application code. The core crate shouldn't
	/// use this and instead should create a new enum variant in [`CoreError`].
	#[error(transparent)]
	Other(#[from] anyhow::Error),
}

impl From<plugin::Error> for ErrorKind {
	fn from(value: plugin::Error) -> Self {
		ErrorKind::Core(CoreError::from(value))
	}
}

impl From<ron::Error> for ErrorKind {
	fn from(value: ron::Error) -> Self {
		ErrorKind::Serde(SerdeError::from(value))
	}
}

impl From<ron::de::SpannedError> for ErrorKind {
	fn from(value: ron::de::SpannedError) -> Self {
		ErrorKind::Serde(SerdeError::from(ron::Error::from(value)))
	}
}

impl From<toml::de::Error> for ErrorKind {
	fn from(value: toml::de::Error) -> Self {
		ErrorKind::Serde(SerdeError::Toml(TomlError::from(value)))
	}
}

impl From<toml::ser::Error> for ErrorKind {
	fn from(value: toml::ser::Error) -> Self {
		ErrorKind::Serde(SerdeError::Toml(TomlError::from(value)))
	}
}

impl From<std::io::Error> for ErrorKind {
	fn from(value: std::io::Error) -> Self {
		ErrorKind::Std(StdError::from(value))
	}
}

impl From<ffi::NulError> for ErrorKind {
	fn from(value: ffi::NulError) -> Self {
		ErrorKind::Std(StdError::from(value))
	}
}

impl From<SystemTimeError> for ErrorKind {
	fn from(value: SystemTimeError) -> Self {
		ErrorKind::Std(StdError::from(value))
	}
}

impl From<str::Utf8Error> for ErrorKind {
	fn from(value: str::Utf8Error) -> Self {
		ErrorKind::Std(StdError::from(value))
	}
}

/// The name of the directory responsible for storing the application's data
/// such as cache. The directory is created at `%AppData%` on Windows.
pub const DATA_DIR: &str = "exanima-modding-toolkit";

/// The name of the directory responsible for storing caches of the application
/// such as instance history. This is a child of `emcore::DATA_DIR`.
pub const CACHE_DIR: &str = "cache";

/// The name of the directory responsible for storing logs of the application.
/// This is a child of `emcore::DATA_DIR`.
pub const LOG_DIR: &str = "log";

#[instrument(level = "trace")]
pub async fn ensure_dir(dir: &Path) -> io::Result<()> {
	if !dir.is_dir() {
		fs::create_dir_all(dir).await?;
	}

	Ok(())
}

/// Returns the path to the application's data directory.
#[instrument(level = "trace")]
pub fn data_dir() -> Option<PathBuf> {
	dirs::data_dir().map(|p| p.join(DATA_DIR))
}

/// Returns the path to the application's cache directory. This is a child
/// of `emcore::DATA_DIR_NAME`.
#[instrument(level = "trace")]
pub async fn cache_dir() -> io::Result<PathBuf> {
	let cache_dir = data_dir()
		.map(|p| p.join(CACHE_DIR))
		.ok_or(io::Error::from(io::ErrorKind::NotFound))?;

	ensure_dir(&cache_dir).await?;

	Ok(cache_dir)
}

/// Returns the path to the application's log directory. This is a child
/// of `emcore::DATA_DIR_NAME`.
#[instrument(level = "trace")]
pub async fn log_dir() -> io::Result<PathBuf> {
	let log_dir = data_dir()
		.map(|p| p.join(LOG_DIR))
		.ok_or(io::Error::from(io::ErrorKind::NotFound))?;

	ensure_dir(&log_dir).await?;

	Ok(log_dir)
}
