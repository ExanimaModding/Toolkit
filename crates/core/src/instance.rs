use std::{path::PathBuf, process, str::FromStr};

use bon::Builder;
use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};
use tokio::{
	fs,
	io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
};
use tracing::{error, info, warn};

use crate::prelude::*;

pub mod prelude {
	pub use crate::instance::{self, Instance, InstanceHistory};
}

pub mod error {
	use crate::prelude::*;

	#[derive(Debug, thiserror::Error)]
	pub enum Build {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
		#[error("{0}")]
		Locked(&'static str),
		#[error("{0}")]
		ProfileBuilder(#[from] profile::error::Builder),
		#[error("{0}")]
		ProfileBuild(#[from] profile::error::Build),
		#[error("{0}")]
		TomlDeserialize(#[from] crate::error::TomlDeserialize),
	}

	#[derive(Debug, thiserror::Error)]
	pub enum Builder {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
	}

	#[derive(Debug, thiserror::Error)]
	pub enum Settings {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
		#[error("{0}")]
		TomlSerialize(#[from] crate::error::TomlSerialize),
	}
}

pub type InstanceHistory = Vec<PathBuf>;

/// Return a result to the instance history
pub async fn history() -> Result<InstanceHistory, crate::error::RonFile> {
	let history_file_path = crate::cache_dir()
		.await
		.map_err(|source| crate::error::Io {
			message: "failed to create cache directory",
			source,
		})?
		.join(Instance::HISTORY_CACHE_RON);

	let file = if history_file_path.is_file() {
		let file = fs::File::open(&history_file_path)
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to open instance history file",
				source,
			})?;
		info!("instance history file opened");
		file
	} else {
		let file = fs::File::create_new(history_file_path)
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create new instance history file",
				source,
			})?;
		info!("instance history file created");
		file
	};

	let mut reader = io::BufReader::new(file);
	let mut buffer = String::new();
	reader
		.read_to_string(&mut buffer)
		.await
		.map_err(|source| crate::error::Io {
			message: "failed to read into buffer for instance history",
			source,
		})?;
	info!("instance history file read into buffer");
	let instance_history = ron::from_str(&buffer).map_err(|source| crate::error::Ron {
		message: "failed to deserialize instance history from buffer",
		source: source.into(),
	})?;
	info!("instance history deserialized from buffer");

	Ok(instance_history)
}

pub async fn write_instance_history(
	instance_history: &[PathBuf],
) -> Result<(), crate::error::RonFile> {
	let history_file_path = crate::cache_dir()
		.await
		.map_err(|source| crate::error::Io {
			message: "failed to create cache directory",
			source,
		})?
		.join(Instance::HISTORY_CACHE_RON);

	let file = fs::File::create(history_file_path)
		.await
		.map_err(|source| crate::error::Io {
			message: "failed to create instance history file",
			source,
		})?;
	let mut writer = io::BufWriter::new(file);
	let buffer = ron::ser::to_string_pretty(instance_history, ron::ser::PrettyConfig::default())
		.map_err(|source| crate::error::Ron {
			message: "failed to serialize instance history into buffer",
			source,
		})?;
	writer
		.write_all(buffer.as_bytes())
		.await
		.map_err(|source| crate::error::Io {
			message: "failed to write into instance history file",
			source,
		})?;
	writer.flush().await.map_err(|source| crate::error::Io {
		message: "failed to flush buffer into instance history file",
		source,
	})?;
	info!("instance history recorded to file");

	Ok(())
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
	pub name: Option<String>,
}

/// An instance is a structure for managing a game directory of Exanima. A user
/// can have multiple copies of Exanima game directories on their system that
/// may be different versions of the game with different mods. The `Instance`
/// implements compatibility for this use case and one can be built from any
/// compatible game directory of Exanima.
#[derive(Debug, Clone, Builder, Getters, MutGetters)]
#[builder(start_fn(vis = ""))]
#[builder(finish_fn(name = build_internal, vis = ""))]
pub struct Instance {
	/// Full path to the instance which also points to the root of the game directory
	#[getset(get = "pub")]
	#[builder(start_fn)]
	path: PathBuf,

	#[builder(field)]
	pub force: bool,

	/// Profile the instance is currently using
	#[getset(get = "pub", get_mut = "pub")]
	#[builder(skip)]
	profile: Profile,

	/// Settings of the instance from `Instance::TOML`
	#[getset(get = "pub")]
	#[builder(skip)]
	settings: Settings,
}

impl Instance {
	/// The name of the file responsible for storing instance specific settings
	/// or information such as the display name of the instance. This is a child of
	/// `Instance::DATA_DIR`.
	pub const TOML: &str = "instance.toml";

	/// The name of the file responsible for executing the game. This is a child of
	/// the root game directory.
	pub const BINARY: &str = "Exanima.exe";

	/// The name of the file responsible for keeping track of the full path to
	/// previously used instances. This is a child of `emcore::CACHE_DIR`.
	pub const HISTORY_CACHE_RON: &str = "instance_history.ron";

	/// The name of the directory responsible for storing the instance's data such
	/// as profiles, caches, etc. The directory is created at the root of the game
	/// directory.
	pub const DATA_DIR: &str = ".emtk";

	/// The name of the file responsible for storing the process ID currently using
	/// the instance. If the lock file is missing or the process with the ID can not be
	/// found, it is safe to use the instance. This is a child of `Instance::DATA_DIR`.
	pub const LOCK: &str = "instance.lock";

	/// The name of the directory that will be iterated over to discover mods. This
	/// is a sibling directory of `Instance::DATA_DIR`.
	pub const MODS_DIR: &str = "mods";

	/// The name of the directory that will store a plugin's assets including game
	/// assets such as rpks within `Instance::PACKAGES_DIR`. This can be found in `Instance::MODS_DIR` nested within a
	/// plugin's folder.
	pub const ASSETS_DIR: &str = "assets";

	/// The name of the directory that will store all of a plugin's game assets.
	/// This is a child directory of `Instance::ASSETS_DIR`.
	pub const PACKAGES_DIR: &str = "packages";

	/// The name of the directory responsible for storing the caches of the
	/// instance such as recent profile path, cache builds, etc. This is a
	/// child of `Instance::DATA_DIR`.
	pub const CACHE_DIR: &str = "cache";

	/// The name of the directory responsible for storing package builds generated
	/// from the mod loader. This is a child of `Instance::CACHE_DIR`.
	pub const CACHE_BUILD_DIR: &str = "build";

	/// The name of the file responsible for tracking the full path to the recent
	/// profile of this instance. This is a child of `Instance::CACHE_DIR`.
	pub const RECENT_PROFILE_RON: &str = "recent_profile.ron";

	/// The name of the directory responsible for storing all of the instance's
	/// profiles. This is a child directory of `Instance::DATA_DIR`.
	pub const PROFILES_DIR: &str = "profiles";

	/// The default name given to a directory used as a profile for the instance
	/// responsible for storing files of the profile's data such as mod load order.
	/// This is a child directory of `Instance::PROFILES_DIR`.
	pub const DEFAULT_PROFILE_DIR: &str = "Default";

	/// The builder of `Instance` and is used to compute if the path points to the
	/// root game directory of a compatible Exanima install
	///
	/// # Examples
	///
	/// ```rust
	/// use emcore::prelude::*;
	///
	/// let maybe_instance = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima";
	/// let instance_builder = Instance::with_path(maybe_instance).unwrap();
	/// ```
	///
	/// # Errors
	///
	/// This function will return an error if:
	///
	/// - `path` doesn't exist
	/// - **Exanima.exe** does not exist within the directory `path` points to
	/// - another process locked the instance and is currently using it
	pub fn with_path(path: impl Into<PathBuf>) -> Result<InstanceBuilder, error::Builder> {
		let path: PathBuf = path.into();
		let path = path.canonicalize().map_err(|source| crate::error::Io {
			message: "failed to find path for instance",
			source,
		})?;
		path.join("Exanima.exe")
			.canonicalize()
			.map_err(|source| crate::error::Io {
				message: "failed to find game executable file",
				source,
			})?;

		Ok(Self::builder(path))
	}

	/// Attempt writing the instance's current profile path to the
	/// `Instance::CACHE_PROFILE_PATH_NAME` file, effectively caching the path to disk.
	///
	/// # Examples
	///
	/// ```rust
	/// use emcore::prelude::*;
	///
	/// let maybe_instance = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima";
	/// let instance = Instance::with_path(maybe_instance).unwrap().build().unwrap();
	/// let new_profile = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima\\.emtk\\profiles\\New";
	/// instance.set_profile(new_profile).unwrap();
	/// ```
	///
	/// # Errors
	///
	/// Currently, an error is returned when attempting to write to a cache. The
	/// profile will still be set if an error occurs.
	///
	/// Errors may be returned according to:
	///
	/// - `ron::ser::to_string`
	/// - `Instance::create_cache_dir`
	/// - `tokio::fs::File::open`
	/// - `tokio::fs::File::create`
	/// - `tokio::io::BufWriter::write_all`
	/// - `tokio::io::BufWriter::flush`
	pub async fn set_profile(
		&mut self,
		profile: Profile,
	) -> Result<&mut Self, crate::error::RonFile> {
		self.profile = profile;

		let buffer =
			ron::ser::to_string(self.profile.path()).map_err(|source| crate::error::Ron {
				message: "failed to serialize path to profile into buffer",
				source,
			})?;

		let file_path = self
			.cache_dir()
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create cache directory",
				source,
			})?
			.join(Self::RECENT_PROFILE_RON);

		let file = match fs::File::create(&file_path).await {
			Ok(file) => file,
			Err(source) => {
				return Err(crate::error::Io {
					message: "failed to create cache file for profile path",
					source,
				})?
			}
		};

		let mut writer = io::BufWriter::new(file);
		writer
			.write_all(buffer.as_bytes())
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to write buffer containing profile path into cache file",
				source,
			})?;
		writer.flush().await.map_err(|source| crate::error::Io {
			message: "failed to flush buffer containing profile path into cache file",
			source,
		})?;
		info!("profile path cached to file");

		Ok(self)
	}

	/// Returns a result from attempting to serialize the given settings to
	/// `Instance::TOML` and then mutating `Instance::settings`.
	///
	/// # Errors
	///
	/// `Instance::settings` will not be mutated if an error occurs.
	///
	/// Errors may be returned according to:
	///
	/// - `Instance::data_dir`
	/// - `tokio::fs::File::create`
	/// - `toml::to_string`
	/// - `tokio::io::BufWriter::write_all`
	/// - `tokio::io::BufWriter::flush`
	pub async fn set_settings(&mut self, settings: Settings) -> Result<&mut Self, error::Settings> {
		let file = fs::File::create(
			self.data_dir()
				.await
				.map_err(|source| crate::error::Io {
					message: "could not find instance data directory",
					source,
				})?
				.join(Self::TOML),
		)
		.await
		.map_err(|source| crate::error::Io {
			message: "failed to create instance settings file",
			source,
		})?;
		info!("instance settings file created");
		let buffer = toml::to_string(&settings).map_err(|source| crate::error::TomlSerialize {
			message: "failed to serialize instance settings into buffer",
			source,
		})?;
		info!("instance settings serialized to buffer");
		let mut writer = io::BufWriter::new(file);
		writer
			.write_all(buffer.as_bytes())
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to write instance settings buffer into file",
				source,
			})?;
		writer.flush().await.map_err(|source| crate::error::Io {
			message: "failed to flush instance settings buffer into file",
			source,
		})?;
		info!("finished writing instance settings to file");

		self.settings = settings;
		Ok(self)
	}

	pub async fn data_dir(&self) -> io::Result<PathBuf> {
		let dir = self.path.join(Self::DATA_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	pub async fn mods_dir(&self) -> io::Result<PathBuf> {
		let dir = self.path.join(Self::MODS_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	/// Return a result to the path of `Instance::PROFILES_DIR`.
	///
	/// To get a list of paths of all profiles for the current instance, see
	/// [`Instance::profile_dirs`].
	pub async fn profiles_dir(&self) -> io::Result<PathBuf> {
		let dir = self.data_dir().await?.join(Self::PROFILES_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	pub async fn cache_dir(&self) -> io::Result<PathBuf> {
		let dir = self.data_dir().await?.join(Self::CACHE_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	pub async fn cache_build_dir(&self) -> io::Result<PathBuf> {
		let dir = self.cache_dir().await?.join(Self::CACHE_BUILD_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	/// Return a result to the path of all the directories contained in
	/// `Instance::PROFILES_DIR` for the current instance.
	///
	/// To get the path to `Instance::PROFILES_DIR`, see
	/// [`Instance::profiles_dir`].
	pub async fn profile_dirs(&self) -> io::Result<Vec<PathBuf>> {
		let profiles_dir = self.profiles_dir().await?;
		let mut profile_paths = Vec::new();
		let mut read_profiles_dir = fs::read_dir(&profiles_dir).await?;
		while let Some(entry) = read_profiles_dir.next_entry().await? {
			let entry_path = entry.path();
			if entry_path.is_dir() {
				profile_paths.push(entry_path);
			};
		}
		Ok(profile_paths)
	}
}

impl<S> InstanceBuilder<S>
where
	S: instance_builder::State,
{
	/// Tells the `InstanceBuilder` to ignore any lock file at the instance path
	/// thus allowing an `Instance` to be built forcefully.
	pub fn force(mut self) -> Self {
		self.force = true;
		self
	}

	pub async fn build(self) -> Result<Instance, error::Build>
	where
		S: instance_builder::IsComplete,
	{
		// TODO: this block works but commenting out for now until all bugs elsewhere are fixed
		// let lock_path = self.path.join(Instance::DATA_DIR).join(Instance::LOCK);
		let force = self.force;
		// if !force && lock_path.is_file() {
		// 	let file = fs::File::open(lock_path).await.unwrap();
		// 	let mut reader = io::BufReader::new(file);
		// 	let mut buffer = String::new();
		// 	reader.read_line(&mut buffer).await.unwrap();
		// 	let s = sysinfo::System::new_with_specifics(
		// 		sysinfo::RefreshKind::nothing()
		// 			.with_processes(sysinfo::ProcessRefreshKind::nothing()),
		// 	);
		// 	if let Ok(pid) = sysinfo::Pid::from_str(buffer.trim())
		// 		&& s.process(pid).is_some()
		// 	{
		// 		return Err(error::Build::Locked(
		// 			"the instance at the given path is currently being used by another process",
		// 		));
		// 	};
		// }

		let mut instance = self.build_internal();
		instance.force = force;

		let data_dir = instance
			.data_dir()
			.await
			.map_err(|source| crate::error::Io {
				message: "could not find instance data directory",
				source,
			})?;

		let file = fs::File::create(data_dir.join(Instance::LOCK))
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create lock file for instance",
				source,
			})?;
		info!("instance's lock file created");
		let mut writer = io::BufWriter::new(file);
		writer
			.write_all(process::id().to_string().as_bytes())
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to write process ID to instance's lock file",
				source,
			})?;
		writer.flush().await.map_err(|source| crate::error::Io {
			message: "failed to flush buffer into instance's lock file",
			source,
		})?;
		info!("process id recorded to instance's lock file");

		let settings_path = data_dir.join(Instance::TOML);
		instance.settings = if settings_path.is_file() {
			let file = fs::File::open(settings_path)
				.await
				.map_err(|source| crate::error::Io {
					message: "failed to open instance settings file",
					source,
				})?;
			info!("instance settings file opened");
			let mut reader = io::BufReader::new(file);
			let mut buffer = String::new();
			reader
				.read_to_string(&mut buffer)
				.await
				.map_err(|source| crate::error::Io {
					message: "failed to read instance settings into buffer",
					source,
				})?;
			info!("instance settings file read into buffer");
			let settings =
				toml::from_str(&buffer).map_err(|source| crate::error::TomlDeserialize {
					message: "failed to deserialize instance settings from buffer",
					source,
				})?;
			info!("instance settings deserialized from buffer");
			settings
		} else {
			info!("instance settings file not found, using default instance settings");
			Settings::default()
		};

		let cache_dir = instance
			.cache_dir()
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create cache directory",
				source,
			})?;
		let recent_profile_cache_path = cache_dir.join(Instance::RECENT_PROFILE_RON);
		let profiles_dir = instance
			.profiles_dir()
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create profiles directory",
				source,
			})?;

		let default_profile_dir = profiles_dir.join(Instance::DEFAULT_PROFILE_DIR);
		let profile_dir = {
			// TODO: refactor if else nesting?
			if let Ok(file) = fs::File::open(&recent_profile_cache_path).await {
				info!("cache file of profile path opened");
				let mut reader = io::BufReader::new(file);
				let mut buffer = String::new();

				if reader.read_to_string(&mut buffer).await.is_ok() {
					match ron::from_str::<String>(&buffer) {
						Ok(contents) => {
							let maybe_valid_path = PathBuf::from(contents);
							if let Some(parent) = maybe_valid_path.parent()
								&& parent == profiles_dir
							{
								info!("cached profile path is valid");
								maybe_valid_path
							} else {
								warn!("cached profile path is invalid, using default path instead");
								default_profile_dir.clone()
							}
						}
						Err(_) => {
							warn!("failed to deserialize cached profile path from buffer, using default path instead");
							default_profile_dir.clone()
						}
					}
				} else {
					warn!("failed to read cached profile path into buffer, using default path instead");
					default_profile_dir.clone()
				}
			} else {
				warn!("failed to open cache file for profile path, using default path instead");
				default_profile_dir.clone()
			}
		};

		if let Err(e) = instance
			.set_profile(Profile::with_path(&profile_dir).await?.build().await?)
			.await
		{
			error!("{}", e);
		};

		// attempt to record this instance to a history file
		{
			let Ok(instance_history) = history().await.or_else(|e| match e {
				crate::error::RonFile::Ron(e) => {
					warn!(e.message);
					Ok(InstanceHistory::new())
				}
				crate::error::RonFile::Io(e) => {
					warn!(e.message);
					Err(e.source)
				}
			}) else {
				return Ok(instance);
			};
			let mut instance_history = if !instance_history.is_empty() {
				// deduplicate any old paths
				instance_history
					.into_iter()
					.filter(|history_path| *history_path != instance.path)
					.collect()
			} else {
				instance_history
			};
			instance_history.push(instance.path.clone());

			if let Err(e) = write_instance_history(&instance_history).await {
				match e {
					crate::error::RonFile::Ron(e) => {
						warn!(e.message);
					}
					crate::error::RonFile::Io(e) => {
						warn!(e.message);
					}
				}
				return Ok(instance);
			};
		}

		Ok(instance)
	}
}

#[cfg(test)]
mod tests {
	use tempfile::{tempdir, NamedTempFile, TempDir};

	use crate::prelude::*;

	/// Prevent `instance::Builder::Io` error when calling `Instance::with_path()`
	fn dummy_exanima_exe(tempdir: &TempDir) -> NamedTempFile {
		tempfile::Builder::new()
			.prefix("Exanima")
			.rand_bytes(0)
			.suffix(".exe")
			.keep(true)
			.tempfile_in(tempdir)
			.unwrap()
	}

	#[tokio::test]
	async fn initialize_empty_instance() {
		let cwd = tempdir().unwrap();
		dummy_exanima_exe(&cwd);

		let instance = Instance::with_path(cwd.path())
			.unwrap()
			.build()
			.await
			.unwrap();
		let data_dir = instance.path.join(Instance::DATA_DIR);
		let cache_dir = data_dir.join(Instance::CACHE_DIR);

		assert!(crate::cache_dir()
			.await
			.unwrap()
			.join(Instance::HISTORY_CACHE_RON)
			.is_file());
		assert!(data_dir.is_dir());
		assert!(instance.path.join(Instance::MODS_DIR).is_dir());
		assert!(cache_dir.is_dir());
		assert!(!cache_dir.join(Instance::CACHE_BUILD_DIR).is_dir());
		assert!(cache_dir.join(Instance::RECENT_PROFILE_RON).is_file());
		assert!(data_dir.join(Instance::PROFILES_DIR).is_dir());
		assert!(instance.profile.path().is_dir());
		assert!(instance
			.profile
			.path()
			.join(Profile::LOAD_ORDER_TOML)
			.is_file());
	}

	#[test]
	fn invalid_instance_path() {
		let cwd = tempdir().unwrap();

		assert!(matches!(
			Instance::with_path("").err().unwrap(),
			instance::error::Builder::Io(crate::error::Io {
				message: _,
				source: _
			})
		));
		assert!(matches!(
			Instance::with_path(cwd.path()).err().unwrap(),
			instance::error::Builder::Io(crate::error::Io {
				message: _,
				source: _
			})
		));
	}

	#[tokio::test]
	async fn invalid_instance_build() {
		let cwd = tempdir().unwrap();
		dummy_exanima_exe(&cwd);

		let profile = Profile::with_path(cwd.path())
			.await
			.unwrap()
			.build()
			.await
			.unwrap();
		let mut instance = Instance::with_path(cwd.path())
			.unwrap()
			.build()
			.await
			.unwrap();
	}
}
