use std::{fmt, path::PathBuf, process};

use bon::Builder;
use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};
use tokio::{fs, io};
use tracing::{error, info, instrument, warn};

use crate::{Error, Result, TomlError, prelude::*};

pub mod prelude {
	pub use crate::instance::{self, Instance};
}

pub type InstanceHistory = Vec<PathBuf>;

/// Return a result to the instance history
#[instrument(level = "trace")]
pub async fn history() -> Result<InstanceHistory> {
	let history_file_path = crate::cache_dir()
		.await
		.map_err(Error::msg("failed to create cache directory"))?
		.join(Instance::HISTORY_CACHE_RON);

	if !history_file_path.is_file() {
		fs::File::create_new(&history_file_path)
			.await
			.map_err(Error::msg("failed to create new instance history file"))?;
		info!("instance history file created");
	}
	let buffer = fs::read_to_string(history_file_path)
		.await
		.map_err(Error::msg(
			"failed to read into buffer for instance history",
		))?;
	info!("instance history file read into buffer");
	let instance_history = ron::from_str(&buffer)
		.map_err(ron::Error::from)
		.map_err(Error::msg(
			"failed to deserialize instance history from buffer",
		))?;
	info!("instance history deserialized from buffer");

	Ok(instance_history)
}

#[instrument(level = "trace")]
pub async fn write_instance_history(instance_history: &[PathBuf]) -> Result<()> {
	let history_file_path = crate::cache_dir()
		.await
		.map_err(Error::msg("failed to create cache directory"))?
		.join(Instance::HISTORY_CACHE_RON);

	let buffer = ron::ser::to_string_pretty(instance_history, ron::ser::PrettyConfig::default())
		.map_err(Error::msg(
			"failed to serialize instance history into buffer",
		))?;
	let _ = fs::write(history_file_path, buffer)
		.await
		.map_err(Error::msg("failed to write into instance history file"));
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
#[builder(derive(Debug))]
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
	#[instrument(level = "trace")]
	pub fn with_path<P: Into<PathBuf> + fmt::Debug>(path: P) -> Result<InstanceBuilder> {
		let path: PathBuf = path.into();
		let path = path
			.canonicalize()
			.map_err(Error::msg("failed to find path for instance"))?;
		path.join("Exanima.exe")
			.canonicalize()
			.map_err(Error::msg("failed to find game executable file"))?;

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
	/// # tokio_test::block_on(async {
	/// let maybe_instance = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima";
	/// let mut instance = Instance::with_path(maybe_instance).unwrap().build().await.unwrap();
	/// let new_profile = Profile::with_path(
	///     "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima\\.emtk\\profiles\\New",
	/// )
	/// .await
	/// .unwrap()
	/// .build()
	/// .await
	/// .unwrap();
	/// instance.set_profile(new_profile).await.unwrap();
	/// # })
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
	/// - `Instance::cache_dir`
	/// - `tokio::fs::write`
	#[instrument(level = "trace")]
	pub async fn set_profile(&mut self, profile: Profile) -> Result<&mut Self> {
		self.profile = profile;

		let buffer = ron::ser::to_string(self.profile.path()).map_err(Error::msg(
			"failed to serialize path to profile into buffer",
		))?;
		info!("profile path serialized into buffer");

		let file_path = self
			.cache_dir()
			.await
			.map_err(Error::msg("failed to create cache directory"))?
			.join(Self::RECENT_PROFILE_RON);

		fs::write(file_path, buffer).await.map_err(Error::msg(
			"failed to write buffer containing profile path into cache file",
		))?;
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
	/// - `toml::to_string`
	/// - `Instance::data_dir`
	/// - `tokio::fs::write`
	#[instrument(level = "trace")]
	pub async fn set_settings(&mut self, settings: Settings) -> Result<&mut Self> {
		let buffer = toml::to_string(&settings)
			.map_err(TomlError::from)
			.map_err(Error::msg(
				"failed to serialize instance settings into buffer",
			))?;
		info!("instance settings serialized to buffer");
		fs::write(
			self.data_dir()
				.await
				.map_err(Error::msg("could not find instance data directory"))?,
			buffer,
		)
		.await
		.map_err(Error::msg(
			"failed to write instance settings buffer into file",
		))?;
		info!("finished writing instance settings to file");

		self.settings = settings;
		Ok(self)
	}

	#[instrument(level = "trace")]
	pub async fn data_dir(&self) -> io::Result<PathBuf> {
		let dir = self.path.join(Self::DATA_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	#[instrument(level = "trace")]
	pub async fn mods_dir(&self) -> io::Result<PathBuf> {
		let dir = self.path.join(Self::MODS_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	/// Return a result to the path of `Instance::PROFILES_DIR`.
	///
	/// To get a list of paths of all profiles for the current instance, see
	/// [`Instance::profile_dirs`].
	#[instrument(level = "trace")]
	pub async fn profiles_dir(&self) -> io::Result<PathBuf> {
		let dir = self.data_dir().await?.join(Self::PROFILES_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	#[instrument(level = "trace")]
	pub async fn cache_dir(&self) -> io::Result<PathBuf> {
		let dir = self.data_dir().await?.join(Self::CACHE_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	#[instrument(level = "trace")]
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
	#[instrument(level = "trace")]
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
	#[instrument(level = "trace")]
	pub fn force(mut self) -> Self {
		self.force = true;
		self
	}

	#[instrument(level = "trace")]
	pub async fn build(self) -> Result<Instance>
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
			.map_err(Error::msg("could not find instance data directory"))?;

		fs::write(
			data_dir.join(Instance::LOCK),
			process::id().to_string().as_bytes(),
		)
		.await
		.map_err(Error::msg(
			"failed to write process ID to instance's lock file",
		))?;
		info!("process ID recorded to instance's lock file");

		let settings_path = data_dir.join(Instance::TOML);
		instance.settings = if settings_path.is_file() {
			let buffer = fs::read_to_string(settings_path)
				.await
				.map_err(Error::msg("failed to read instance settings into buffer"))?;
			info!("instance settings file read into buffer");
			let settings = toml::from_str(&buffer)
				.map_err(TomlError::from)
				.map_err(Error::msg(
					"failed to deserialize instance settings from buffer",
				))?;
			info!("instance settings deserialized from buffer");
			settings
		} else {
			info!("instance settings file not found, using default instance settings");
			Settings::default()
		};

		let cache_dir = instance
			.cache_dir()
			.await
			.map_err(Error::msg("failed to create cache directory"))?;
		let recent_profile_cache_path = cache_dir.join(Instance::RECENT_PROFILE_RON);
		let profiles_dir = instance
			.profiles_dir()
			.await
			.map_err(Error::msg("failed to create profiles directory"))?;

		let default_profile_dir = profiles_dir.join(Instance::DEFAULT_PROFILE_DIR);
		let default_profile_dir = default_profile_dir.as_path();
		let profile_dir = fs::read_to_string(recent_profile_cache_path)
			.await
			.map(move |buffer| {
				ron::from_str::<String>(&buffer)
					.map(move |maybe_path| {
						let maybe_path = PathBuf::from(maybe_path);
						if let Some(parent) = maybe_path.parent()
							&& parent == profiles_dir
						{
							maybe_path
						} else {
							warn!("cached profile path is invalid, using default path instead");
							default_profile_dir.to_path_buf()
						}
					})
					.map_err(ron::Error::from)
					.map_err(Error::msg(
						"failed to deserialize cached profile path from buffer, using default path instead",
					))
					.map_err(|e| warn!("{e}"))
					.unwrap_or(default_profile_dir.to_path_buf())
			})
			.map_err(Error::msg(
				"failed to read cached profile path into buffer, using default path instead",
			))
			.map_err(|e| warn!("{e}"))
			.unwrap_or(default_profile_dir.to_path_buf());

		let _ = instance
			.set_profile(Profile::with_path(&profile_dir).await?.build().await?)
			.await
			.map_err(|e| error!("{e}"));

		// attempt to record this instance to a history file
		{
			let instance_history = history()
				.await
				.map_err(|e| warn!("{e}"))
				.unwrap_or_default();
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

			let _ = write_instance_history(&instance_history)
				.await
				.map_err(|e| warn!("{e}"));
		}

		Ok(instance)
	}
}

#[cfg(test)]
mod tests {
	use tempfile::{NamedTempFile, TempDir, tempdir};

	use crate::{Error, ErrorKind, prelude::*};

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

		assert!(
			crate::cache_dir()
				.await
				.unwrap()
				.join(Instance::HISTORY_CACHE_RON)
				.is_file()
		);
		assert!(data_dir.is_dir());
		assert!(instance.path.join(Instance::MODS_DIR).is_dir());
		assert!(cache_dir.is_dir());
		assert!(!cache_dir.join(Instance::CACHE_BUILD_DIR).is_dir());
		assert!(cache_dir.join(Instance::RECENT_PROFILE_RON).is_file());
		assert!(data_dir.join(Instance::PROFILES_DIR).is_dir());
		assert!(instance.profile.path().is_dir());
		assert!(
			instance
				.profile
				.path()
				.join(Profile::LOAD_ORDER_TOML)
				.is_file()
		);
	}

	#[test]
	fn invalid_instance_path() {
		let cwd = tempdir().unwrap();

		assert!(matches!(
			Instance::with_path("").err().unwrap(),
			Error {
				kind: ErrorKind::Io { .. },
				message: _,
			},
		));
		assert!(matches!(
			Instance::with_path(cwd.path()).err().unwrap(),
			Error {
				kind: ErrorKind::Io { .. },
				message: _,
			}
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
