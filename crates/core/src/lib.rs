#![feature(let_chains)]
#![feature(error_generic_member_access)]
#![doc = include_str!("../README.md")]
#![deny(clippy::panic)]
#![deny(clippy::expect_used)]
#![deny(clippy::unwrap_used)]

pub mod cache;
pub mod instance;
pub mod plugin;
pub mod profile;

use std::path::{Path, PathBuf};

use tokio::{fs, io};

pub use instance::{Instance, InstanceHistory};
pub use plugin::Plugin;
pub use profile::Profile;

pub mod prelude {
	pub use crate::{instance::prelude::*, plugin::prelude::*, profile::prelude::*};
}

pub mod error {
	use std::time::SystemTimeError;

	use tokio::io;

	#[derive(Debug, thiserror::Error)]
	#[error("{0}")]
	pub struct InvalidUnicode(&'static str);

	#[derive(Debug, thiserror::Error)]
	#[error("{message}")]
	pub struct Io {
		pub message: &'static str,
		#[backtrace]
		pub source: io::Error,
	}

	#[derive(Debug, thiserror::Error)]
	#[error("{0}")]
	pub struct MissingDll(pub &'static str);

	#[derive(Debug, thiserror::Error)]
	#[error("{message}")]
	pub struct Ron {
		pub message: &'static str,
		#[backtrace]
		pub source: ron::Error,
	}

	#[derive(Debug, thiserror::Error)]
	pub enum RonFile {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
		#[error("{0}")]
		Ron(#[from] Ron),
	}

	#[derive(Debug, thiserror::Error)]
	#[error("{message}")]
	pub struct Time {
		pub message: &'static str,
		#[backtrace]
		pub source: SystemTimeError,
	}

	#[derive(Debug, thiserror::Error)]
	#[error("{message}")]
	pub struct TomlDeserialize {
		pub message: &'static str,
		#[backtrace]
		pub source: toml::de::Error,
	}

	#[derive(Debug, thiserror::Error)]
	pub enum TomlDeFile {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
		#[error("{0}")]
		TomlDeserialize(#[from] TomlDeserialize),
	}

	#[derive(Debug, thiserror::Error)]
	#[error("{message}")]
	pub struct TomlSerialize {
		pub message: &'static str,
		#[backtrace]
		pub source: toml::ser::Error,
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0}")]
	ExpectedDataDir(&'static str),
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

pub async fn ensure_dir(dir: &Path) -> Result<(), io::Error> {
	if !dir.is_dir() {
		fs::create_dir_all(dir).await?;
	}

	Ok(())
}

/// Returns the path to the application's data directory.
pub fn data_dir() -> Option<PathBuf> {
	dirs::data_dir().map(|p| p.join(DATA_DIR))
}

/// Returns the path to the application's cache directory. This is a child
/// of `emcore::DATA_DIR_NAME`.
pub async fn cache_dir() -> Result<PathBuf, io::Error> {
	let cache_dir = data_dir()
		.map(|p| p.join(CACHE_DIR))
		.ok_or(io::Error::from(io::ErrorKind::NotFound))?;

	ensure_dir(&cache_dir).await?;

	Ok(cache_dir)
}

/// Returns the path to the application's log directory. This is a child
/// of `emcore::DATA_DIR_NAME`.
pub async fn log_dir() -> Result<PathBuf, io::Error> {
	let log_dir = data_dir()
		.map(|p| p.join(LOG_DIR))
		.ok_or(io::Error::from(io::ErrorKind::NotFound))?;

	ensure_dir(&log_dir).await?;

	Ok(log_dir)
}
