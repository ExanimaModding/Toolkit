use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Deserializer};

pub const METADATA_RON: &str = "metadata.ron";

/// A HashMap with the key as the canonicalized path to a file and the value as
/// the file's "date modified" metadata in unix timestamp represented in u64
pub type Metadata = HashMap<PathBuf, u64>;

/// `PathBuf` can not be deserialized from ron so we deserialize the path as a
/// `String` then map it into `PathBuf`
pub fn deserialize_metadata<'de, D>(deserializer: D) -> Result<Metadata, D::Error>
where
	D: Deserializer<'de>,
{
	let raw_metadata: HashMap<String, u64> = HashMap::deserialize(deserializer)?;

	let metadata = raw_metadata
		.into_iter()
		.map(|(key, value)| (PathBuf::from(key), value))
		.collect();

	Ok(metadata)
}

#[cfg(test)]
mod tests {
	use std::{
		fs,
		io::{self, Read, Write},
		time::UNIX_EPOCH,
	};

	use pretty_assertions::assert_eq;
	use tempfile::{tempdir, Builder};

	use crate::cache::deserialize_metadata;

	#[test]
	fn new_metadata() {
		let dir = tempdir().unwrap();
		let mut file = Builder::new()
			.prefix("emtk-metadata")
			.rand_bytes(0)
			.suffix(".ron")
			.keep(true)
			.tempfile_in(&dir)
			.unwrap();
		let file_path = dir.path().join("emtk-metadata.ron").canonicalize().unwrap();
		let time = file
			.as_file()
			.metadata()
			.unwrap()
			.modified()
			.unwrap()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs();

		let mut metadata = crate::cache::Metadata::new();
		metadata.insert(file_path.clone(), time);

		let buffer =
			ron::ser::to_string_pretty(&metadata, ron::ser::PrettyConfig::default()).unwrap();
		write!(file, "{}", buffer).unwrap();

		let mut reader = io::BufReader::new(fs::File::open(&file_path).unwrap());
		let mut buffer = String::new();
		reader.read_to_string(&mut buffer).unwrap();
		let metadata =
			deserialize_metadata(&mut ron::de::Deserializer::from_str(&buffer).unwrap()).unwrap();

		assert_eq!(
			metadata.get(&file_path.canonicalize().unwrap()).unwrap(),
			&time
		);
	}
}
