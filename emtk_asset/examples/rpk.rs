use deku::prelude::*;
use emtk_asset::Format;
use std::{env, fs, io, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut file_path = PathBuf::from(env::var("EXANIMA_EXE")?);
	file_path.pop();
	file_path.push("Textures.rpk");

	// Deserializing
	let mut file = fs::File::open(&file_path)?;
	let mut buf_reader = io::BufReader::new(&mut file);
	let mut reader = Reader::new(&mut buf_reader);
	let format = Format::from_reader_with_ctx(&mut reader, ())?;

	// Serializing
	file_path.pop();
	file_path.push("Custom.rpk");
	let mut file = fs::File::create(&file_path)?;
	let mut buf_writer = io::BufWriter::new(&mut file);
	let mut writer = Writer::new(&mut buf_writer);
	format.to_writer(&mut writer, ())?;

	Ok(())
}
