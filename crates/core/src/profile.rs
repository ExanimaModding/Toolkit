use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	time::UNIX_EPOCH,
};

use bon::Builder;
use getset::Getters;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio::{
	fs,
	io::{self, AsyncReadExt, AsyncWriteExt},
};
use tracing::{info, warn};

use crate::prelude::*;

pub mod prelude {
	pub use crate::profile::{self, Profile};
}

pub mod error {
	#[derive(Debug, thiserror::Error)]
	pub enum Build {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
		#[error("{0}")]
		ParentDir(#[from] ParentDir),
		#[error("{0}")]
		TomlSerialize(#[from] crate::error::TomlSerialize),
	}

	#[derive(Debug, thiserror::Error)]
	pub enum Builder {
		#[error("{0}")]
		LoadOrder(#[from] LoadOrderDe),
		#[error("{0}")]
		Io(#[from] crate::error::Io),
	}

	#[derive(Debug, thiserror::Error)]
	pub enum CacheBuildCheck {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
		#[error("{0}")]
		ParentDir(#[from] ParentDir),
		#[error("{0}")]
		RonFile(#[from] crate::error::RonFile),
		#[error("{0}")]
		Time(#[from] crate::error::Time),
	}

	#[derive(Debug, thiserror::Error)]
	#[error("{0}")]
	pub struct InvalidParentDir(pub &'static str);

	#[derive(Debug, thiserror::Error)]
	pub enum LoadOrderDe {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
		#[error("{0}")]
		TomlDeserialize(#[from] crate::error::TomlDeserialize),
	}

	#[derive(Debug, thiserror::Error)]
	pub enum LoadOrderSer {
		#[error("{0}")]
		Io(#[from] crate::error::Io),
		#[error("{0}")]
		TomlSerialize(#[from] crate::error::TomlSerialize),
	}

	#[derive(Debug, thiserror::Error)]
	pub enum ParentDir {
		#[error("{0}")]
		InvalidParentDir(#[from] InvalidParentDir),
		#[error("{0}")]
		Io(#[from] crate::error::Io),
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct LoadOrderEntry {
	pub enabled: bool,
	pub priority: u32,
	#[serde(skip)]
	pub display_name: Option<String>,
	#[serde(skip)]
	pub version: Option<String>,
}

impl LoadOrderEntry {
	pub fn new(
		enabled: bool,
		priority: u32,
		display_name: Option<String>,
		version: Option<String>,
	) -> Self {
		Self {
			enabled,
			priority,
			display_name,
			version,
		}
	}
}

pub type LoadOrder = HashMap<plugin::Id, LoadOrderEntry>;

// TODO: add backup support for profile directories
// - zip and archive entire profile directory into "{`Instance::DATA_DIR`}/backups"
// - filename of backup is timestamp along with name of profile directory
/// `Profile` is built from data inside a directory that contains:
///
/// - load order and enabled/disabled status of mods.
/// - game saves and settings (normally in %AppData%\Exanima)
///
/// There should be at least one profile called *Default*. Profiles will be stored in a *profiles*
/// directory.
#[derive(Default, Clone, Debug, Builder, Getters)]
#[builder(state_mod(vis = "pub(crate)"))]
#[builder(start_fn(vis = ""))]
#[builder(finish_fn(name = build_internal, vis = ""))]
pub struct Profile {
	/// Full path to the profile
	#[getset(get = "pub")]
	#[builder(start_fn)]
	path: PathBuf,
	#[getset(get = "pub")]
	#[builder(setters(vis = ""))]
	load_order: LoadOrder,
}

impl Profile {
	/// The name of the directory responsible for storing the caches of the
	/// profile such as cache builds. This is a child of `Profile { ... }.path`.
	pub const CACHE_DIR: &str = "cache";

	/// The name of the directory responsible for storing package builds generated
	/// from the mod loader. This is a child of `Instance::CACHE_DIR`.
	pub const CACHE_BUILD_DIR: &str = "build";

	/// The name of the file responsible for tracking the load order of mods for the
	/// current instance's profile. This is a child of `Instance { ... }.profile.path`.
	pub const LOAD_ORDER_TOML: &str = "load_order.toml";

	/// The name of the file responsible for caching the load order of mods for
	/// the most recent build of the current instance's profile. This is a child of
	/// `Profile::CACHE_BUILD_DIR`.
	pub const LOAD_ORDER_RON: &str = "load_order.ron";

	pub async fn with_path(
		path: impl Into<PathBuf>,
	) -> Result<ProfileBuilder<profile_builder::SetLoadOrder>, error::Builder> {
		let path = path.into();
		crate::ensure_dir(&path)
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create profile directory",
				source,
			})?;
		let path = path.canonicalize().map_err(|source| crate::error::Io {
			message: "failed to canonicalize path to profile",
			source,
		})?;

		let dummy_profile = Self {
			path,
			load_order: LoadOrder::default(),
		};

		let profile_builder = Profile::builder(dummy_profile.path.clone())
			.load_order(dummy_profile.read_load_order().await?);

		info!("profile with path is valid");
		Ok(profile_builder)
	}

	/// Returns a result from attempting to serialize the given load order to
	/// `Profile::LOAD_ORDER_TOML` and then mutating `Profile::load_order`.
	///
	/// # Errors
	///
	/// `Profile::load_order` will not be mutated if an error occurs.
	///
	/// Errors may be returned according to:
	///
	/// - `tokio::fs::File::create`
	/// - `toml::to_string`
	/// - `tokio::io::BufWriter::write_all`
	/// - `tokio::io::BufWriter::flush`
	pub async fn set_load_order(
		&mut self,
		load_order: LoadOrder,
	) -> Result<&mut Self, error::LoadOrderSer> {
		let file = fs::File::create(self.path.join(Self::LOAD_ORDER_TOML))
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create profile's load order file",
				source,
			})?;
		info!("profile's load order file created");
		let buffer =
			toml::to_string(&load_order).map_err(|source| crate::error::TomlSerialize {
				message: "failed to serialize profile's load order into buffer",
				source,
			})?;
		info!("profile's load order serialized to buffer");
		let mut writer = io::BufWriter::new(file);
		writer
			.write_all(buffer.as_bytes())
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to write profile's load order buffer into file",
				source,
			})?;
		writer.flush().await.map_err(|source| crate::error::Io {
			message: "failed to flush profile's load order buffer into file",
			source,
		})?;
		info!("finished writing profile's load order to file");

		self.load_order = load_order;
		Ok(self)
	}

	pub async fn game_dir(&self) -> Result<PathBuf, error::ParentDir> {
		let Some(mods_path) = self.path.ancestors().nth(3) else {
			return Err(error::InvalidParentDir(
				"failed to validate path to instance's game directory",
			))?;
		};

		crate::ensure_dir(mods_path)
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create game directory",
				source,
			})?;

		Ok(mods_path.into())
	}

	pub async fn mods_dir(&self) -> Result<PathBuf, error::ParentDir> {
		let mods_path = self.game_dir().await?.join(Instance::MODS_DIR);

		crate::ensure_dir(&mods_path)
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to create mods directory",
				source,
			})?;

		Ok(mods_path)
	}

	pub async fn cache_dir(&self) -> io::Result<PathBuf> {
		let dir = self.path.join(Self::CACHE_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	pub async fn cache_build_dir(&self) -> io::Result<PathBuf> {
		let dir = self.cache_dir().await?.join(Self::CACHE_BUILD_DIR);
		crate::ensure_dir(&dir).await?;
		Ok(dir)
	}

	/// Return a result to the timestamp of mod files
	pub async fn cache_build_metadata(
		&self,
	) -> Result<crate::cache::Metadata, crate::error::RonFile> {
		let cache_build_dir = self.cache_build_dir().await.map_err(|source| {
			crate::error::RonFile::Io(crate::error::Io {
				message: "failed to create cache build directory",
				source,
			})
		})?;
		let metadata_path = cache_build_dir.join(crate::cache::METADATA_RON);
		if !metadata_path.is_file() {
			return Ok(crate::cache::Metadata::new());
		};
		let file = fs::File::open(metadata_path).await.map_err(|source| {
			crate::error::RonFile::Io(crate::error::Io {
				message: "failed to open cache build metadata file",
				source,
			})
		})?;
		info!("cache build metadata file opened");
		let mut reader = io::BufReader::new(file);
		let mut buffer = String::new();
		reader.read_to_string(&mut buffer).await.map_err(|source| {
			crate::error::RonFile::Io(crate::error::Io {
				message: "failed to read into buffer for cache build metadata",
				source,
			})
		})?;
		info!("cache build metadata read into buffer");
		let metadata = crate::cache::deserialize_metadata(
			&mut ron::de::Deserializer::from_str(&buffer).map_err(|source| {
				crate::error::RonFile::Ron(crate::error::Ron {
					message: "failed to create deserializer for cache build metadata from buffer",
					source: source.into(),
				})
			})?,
		)
		.map_err(|source| {
			crate::error::RonFile::Ron(crate::error::Ron {
				message: "failed to deserialize cache build metadata from buffer",
				source,
			})
		})?;
		info!("cache build metadata deserialized from buffer");

		Ok(metadata)
	}

	/// Return a result to true if a mod hasn't changed according to the metadata
	/// file else return false.
	pub async fn is_cache_build_valid(&self) -> Result<bool, error::CacheBuildCheck> {
		/// Recursion in a mod directory is used to support loose-files.
		async fn is_mod_valid(
			metadata: &mut crate::cache::Metadata,
			dir: &Path,
		) -> Result<bool, error::CacheBuildCheck> {
			let mut read_dir = fs::read_dir(dir).await.map_err(|source| crate::error::Io {
				message: "failed to read mod directory entries",
				source,
			})?;
			while let Some(entry) =
				read_dir
					.next_entry()
					.await
					.map_err(|source| crate::error::Io {
						message: "failed to read next entry in mod directory",
						source,
					})? {
				let entry_path =
					entry
						.path()
						.canonicalize()
						.map_err(|source| crate::error::Io {
							message: "failed to find path to a mod asset",
							source,
						})?;

				if entry_path.is_dir() {
					// recurse into directory
					if !Box::pin(is_mod_valid(metadata, &entry_path)).await? {
						return Ok(false);
					};
					continue;
				}

				if entry_path.is_file() {
					let Some((_, metadata_timestamp)) = metadata.get_key_value(&entry_path) else {
						// path isn't in hashmap. new mod, build cache
						return Ok(false);
					};

					let file_timestamp = fs::metadata(&entry_path).await
						.map_err(|source| crate::error::Io {
							message: "failed to read metadata of mod asset",
							source,
						})?
						.modified()
						.map_err(|source| crate::error::Io {
							message: "failed to get modified date time metadata of mod asset",
							source,
						})?
						.duration_since(UNIX_EPOCH)
						.map_err(|source| crate::error::Time {
							message: "failed to get the unix epoch timestamp of the mod asset's modified date time metadata",
							source,
						})?
						.as_secs();
					if *metadata_timestamp != file_timestamp {
						// either exanima or a mod updated, build cache
						return Ok(false);
					}

					// remove path from metadata to check later if a mod was deleted
					metadata.remove(&entry_path);
				}
			}

			Ok(true)
		}

		let mut metadata = self.cache_build_metadata().await?;

		// check vanilla game files
		let game_dir = self.game_dir().await?;
		let mut read_game_dir =
			fs::read_dir(game_dir)
				.await
				.map_err(|source| crate::error::Io {
					message: "failed to read game directory entries",
					source,
				})?;
		while let Some(entry) =
			read_game_dir
				.next_entry()
				.await
				.map_err(|source| crate::error::Io {
					message: "failed to read next entry in game directory",
					source,
				})? {
			let entry_path = entry.path();
			if entry_path.is_file()
				&& let Some(extension_os) = entry_path.extension()
				&& let Some(extension) = extension_os.to_str()
				&& extension == "rpk"
				&& !is_mod_valid(&mut metadata, &entry_path).await?
			{
				return Ok(false);
			}
		}

		// check mod directories
		let mods_dir = self.mods_dir().await?;
		for (plugin_id, load_order_entry) in &self.load_order {
			if !load_order_entry.enabled {
				info!("mod isn't enabled, skipped \"{}\"", plugin_id);
				continue;
			}
			let mod_dir = mods_dir.join(plugin_id.to_string());
			if !mod_dir.is_dir() {
				warn!("mod isn't a directory, skipped \"{}\"", plugin_id);
				continue;
			}
			let mod_dir = mod_dir.canonicalize().map_err(|source| crate::error::Io {
				message: "failed to find path for mod directory",
				source,
			})?;
			if !is_mod_valid(&mut metadata, &mod_dir).await? {
				return Ok(false);
			};
		}

		// check if any mods are left
		if !metadata.is_empty() {
			return Ok(false);
		}
		Ok(true)
	}

	/// Returns a result to the load order from deserializing the load order file.
	///
	/// # Errors
	///
	/// Errors may be returned according to:
	///
	/// - `tokio::fs::File::open`
	/// - `tokio::fs::File::create_new`
	/// - `tokio::io::BufReader::read_to_string`
	/// - `toml::from_str`
	pub async fn read_load_order(&self) -> Result<LoadOrder, error::LoadOrderDe> {
		let load_order_path = self.path.join(Profile::LOAD_ORDER_TOML);
		let file = if load_order_path.is_file() {
			let file =
				fs::File::open(&load_order_path)
					.await
					.map_err(|source| crate::error::Io {
						message: "failed to open load order file",
						source,
					})?;
			info!("load order file opened");
			file
		} else {
			let file = fs::File::create_new(load_order_path)
				.await
				.map_err(|source| crate::error::Io {
					message: "failed to create new load order file",
					source,
				})?;
			info!("load order file created");
			file
		};
		let mut reader = io::BufReader::new(file);
		let mut buffer = String::new();
		reader
			.read_to_string(&mut buffer)
			.await
			.map_err(|source| crate::error::Io {
				message: "failed to read into buffer for load order",
				source,
			})?;
		info!("load order read into buffer");	
		let mut load_order: Vec<_> = toml::from_str::<LoadOrder>(&buffer)
			.map_err(|source| crate::error::TomlDeserialize {
				message: "failed to deserialize load order",
				source,
			})?
			.into_iter()
			.collect();
		info!("load order deserialized from buffer");

		// ensure removal of gaps in load order priority
		load_order.sort_by(|(_, a), (_, b)| a.priority.cmp(&b.priority));
		let load_order: HashMap<_, _> = load_order
			.into_iter()
			.enumerate()
			.map(|(i, (id, mut entry))| {
				entry.priority = i as _;
				(id, entry)
			})
			.collect();

		Ok(load_order)
	}
}

impl<S> ProfileBuilder<S>
where
	S: profile_builder::State,
{
	pub async fn build(self) -> Result<Profile, error::Build>
	where
		S: profile_builder::IsComplete,
	{
		info!("building profile");
		let mut profile = self.build_internal();

		let discovered_mods = {
			info!("starting mod discovery");

			let mods_path = profile.mods_dir().await?;
			info!("path to instance's mods directory is valid");
			if !mods_path.is_dir() {
				fs::create_dir_all(&mods_path)
					.await
					.map_err(|source| crate::error::Io {
						message: "failed to create mods directory",
						source,
					})?;
				info!("mods directory created");
			}

			let mut discovered_mods = Vec::new();
			for entry in mods_path.read_dir().unwrap().flatten() {
				let entry_path = entry.path();
				let os_string = entry.file_name();
				let Some(entry_name) = os_string.to_str() else {
					warn!("failed to get name from a directory path",);
					continue;
				};
				if !entry_path.is_dir() {
					warn!("not directory, skipping \"{}\"", entry_name);
					continue;
				}
				let Ok(plugin_id) = plugin::Id::try_from(entry_name) else {
					warn!("not valid plugin id, skipping \"{}\"", entry_name);
					continue;
				};

				let manifest_path = entry_path.join(plugin::Manifest::TOML);
				let Ok(file) = fs::File::open(manifest_path).await else {
					warn!(
						"failed to open plugin manifest file, skipping \"{}\"",
						entry_name
					);
					continue;
				};
				let mut reader = io::BufReader::new(file);
				let mut buffer = String::new();
				if reader.read_to_string(&mut buffer).await.is_err() {
					warn!(
						"failed to read plugin manifest file to buffer, skipping \"{}\"",
						entry_name
					);
					continue;
				}
				let Ok(manifest) = toml::from_str::<plugin::Manifest>(&buffer) else {
					warn!(
						"failed to deserialize plugin manifest from buffer, skipping \"{}\"",
						entry_name
					);
					continue;
				};

				if let Some(entry) = profile.load_order.get_mut(&plugin_id) {
					entry.display_name = Some(manifest.plugin.name.clone());
					entry.version = Some(manifest.plugin.version.clone());
				}

				discovered_mods.push((
					plugin_id,
					Some(manifest.plugin.name),
					Some(manifest.plugin.version),
				));
				info!(
					"discovered mod \"{}\"",
					entry.file_name().display().to_string()
				);
			}

			info!("finished discovering mods");
			discovered_mods
		};

		let mut load_order_updated = false;
		if !discovered_mods.is_empty() || !profile.path.join(Profile::LOAD_ORDER_TOML).is_file() {
			if profile.load_order.is_empty() {
				let mut new_load_order = HashMap::new();
				for (i, (plugin_id, display_name, version)) in
					discovered_mods.into_iter().enumerate()
				{
					let plugin_entry = LoadOrderEntry::new(false, i as u32, display_name, version);
					new_load_order.insert(plugin_id, plugin_entry);
				}
				profile.load_order = new_load_order;
				load_order_updated = true;
			} else {
				let new_plugin_ids: Vec<(plugin::Id, Option<String>, Option<String>)> =
					discovered_mods
						.into_iter()
						.filter(|(plugin_id, _, _)| !profile.load_order.contains_key(plugin_id))
						.collect();
				if !new_plugin_ids.is_empty() {
					load_order_updated = true
				}

				for (plugin_id, display_name, version) in new_plugin_ids.into_iter() {
					let id_str = plugin_id.to_string();
					let load_order_entry = LoadOrderEntry::new(
						false,
						(profile.load_order.len() + 1) as u32,
						display_name,
						version,
					);
					profile.load_order.insert(plugin_id, load_order_entry);
					info!(
						"added newly discovered mod to existing load order \"{}\"",
						id_str
					);
				}
			};
		}

		if load_order_updated {
			// TODO: due to delicate procedure, back up file first before writing
			// FIX: consider an alternative to fs::File::create() due to docs mentioning truncating file contents
			// maybe write to a temp file and overwrite original file on success?
			let file = fs::File::create(profile.path.join(Profile::LOAD_ORDER_TOML))
				.await
				.map_err(|source| crate::error::Io {
					message: "failed to create load order file",
					source,
				})?;
			info!("load order file created");
			let buffer = toml::to_string(&profile.load_order).map_err(|source| {
				crate::error::TomlSerialize {
					message: "failed to serialize load order into buffer",
					source,
				}
			})?;
			info!("load order serialized to buffer");

			let mut writer = io::BufWriter::new(file);
			writer
				.write_all(buffer.as_bytes())
				.await
				.map_err(|source| crate::error::Io {
					message: "failed to write load order buffer into file",
					source,
				})?;
			writer.flush().await.map_err(|source| crate::error::Io {
				message: "failed to flush load order buffer into file",
				source,
			})?;
			info!("finished writing update to load order file");
		}

		Ok(profile)
	}
}

// #[cfg(test)]
// mod tests {
// 	use std::{
// 		fs,
// 		io::{self, Write},
// 	};
// 	use tempfile::{tempdir, Builder};

// 	use crate::prelude::*;

// 	#[test]
// 	fn load_order_error() {
// 		let dir = tempdir().unwrap();

// 		let profile = Profile::with_path(dir.path()).unwrap().build().unwrap();
// 		assert!(profile.path.join("load_order.toml").is_file());

// 		let file = fs::File::create(profile.path.join("load_order.toml")).unwrap();
// 		let mut writer = io::BufWriter::new(file);
// 		writer.write_all(&[]).unwrap();

// 		let profile_clone = Profile::with_path(dir.path()).unwrap().build().unwrap();

// 		dbg!(&profile.path);
// 		dbg!(&profile.load_order);
// 	}
// }
