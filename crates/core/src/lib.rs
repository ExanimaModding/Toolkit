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

#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub enum ErrorKind {
	Io {
		#[from]
		source: io::Error,
		backtrace: Backtrace,
	},
	Nul {
		#[from]
		source: ffi::NulError,
		backtrace: Backtrace,
	},
	Ron {
		#[from]
		source: ron::Error,
		backtrace: Backtrace,
	},
	Time {
		#[from]
		source: SystemTimeError,
		backtrace: Backtrace,
	},
	#[error(transparent)]
	Toml(#[from] TomlError),
	#[error(transparent)]
	Other(#[from] anyhow::Error),
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
