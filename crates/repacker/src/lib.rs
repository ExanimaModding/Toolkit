pub mod metadata;
pub mod types;
pub mod utils;

use crate::{types::rpk::RPK, utils::SourceData};
use bitstream_io::{BitRead, BitReader, LittleEndian};
use metadata::MagicBytes;
use std::{fs::File, path::PathBuf};

pub async fn pack(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
	let src_path = PathBuf::from(src);

	let mut handles = vec![];
	let dest = String::from(dest);
	for entry in src_path.read_dir()? {
		let entry = entry?;
		let path = entry.path();

		if !path.is_dir() {
			continue;
		}

		let dest = dest.clone();
		handles.push(tokio::spawn(async move {
			if let Err(e) = RPK::pack(path.to_str().unwrap(), dest.as_str()) {
				eprintln!("Skipping folder at '{}': {}", path.to_str().unwrap(), e);
			};
		}));
	}
	futures::future::join_all(handles).await;

	Ok(())
}

async fn run_unpack(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
	let src_path = PathBuf::from(src);

	let mut reader = BitReader::endian(File::open(&src_path)?, LittleEndian);

	let magic = reader.read::<u32>(32)?;
	let magic = match MagicBytes::try_from(magic) {
		Ok(magic) => magic,
		Err(e) => return Err(Box::new(crate::metadata::Error::InvalidMagic(e))),
	};

	let mut dest_path = PathBuf::from(dest);
	dest_path.push(src_path.with_extension("").file_name().unwrap());

	magic
		.unpack(
			SourceData::Path(String::from(src_path.to_str().unwrap())),
			dest_path.to_str().unwrap(),
		)
		.await?;

	Ok(())
}

pub async fn unpack(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
	let src_path = PathBuf::from(src);

	if src_path.is_file() {
		run_unpack(src, dest).await?;
	}

	if src_path.is_dir() {
		let mut handles = vec![];
		for entry in src_path.read_dir()? {
			let entry = entry?;

			if entry.file_type()?.is_file() {
				let dest_clone = dest.to_owned();

				handles.push(tokio::spawn(async move {
					let path = entry.path();
					let path_str = path.to_str().unwrap();

					if let Err(e) = run_unpack(path_str, &dest_clone).await {
						eprintln!("{}", e)
					};
				}));
			}
		}
		futures::future::join_all(handles).await;
	}

	Ok(())
}
