use deku::prelude::*;
use emtk_asset::Format;
use std::{env, fs, io, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut unknown1 = Vec::new();
	let mut four_cc = Vec::new();
	let mut unknown2 = Vec::new();
	let mut flags = Vec::new();

	let mut file_path = PathBuf::from(env::var("EXANIMA_EXE")?);
	file_path.pop();

	for entry in file_path.read_dir().unwrap().flatten() {
		let entry_path = entry.path();
		if !entry_path.is_file() {
			continue;
		}

		// Deserializing
		let mut file = fs::File::open(&entry_path)?;
		let mut buf_reader = io::BufReader::new(&mut file);
		let mut reader = Reader::new(&mut buf_reader);
		let format = Format::from_reader_with_ctx(&mut reader, ())?;

		if let Format::Rpk(rpk) = &format {
			for data in &rpk.data {
				let mut buf_reader = io::BufReader::new(io::Cursor::new(data));
				let mut reader = Reader::new(&mut buf_reader);
				let format = Format::from_reader_with_ctx(&mut reader, ())?;
				if let Format::Rfi(rfi) = &format {
					unknown1.push(rfi._unknown1);
					four_cc.push(rfi.four_cc);
					unknown2.push(rfi._unknown2);
					flags.push(rfi.flags);
				}
			}
		}
	}

	unknown1.sort();
	unknown1.dedup();
	println!("unknown1: {:#08X?}", unknown1);

	four_cc.sort();
	four_cc.dedup();
	println!("four_cc: {:#08X?}", four_cc);

	unknown2.sort();
	unknown2.dedup();
	println!("unknown2: {:#08X?}", unknown2);

	flags.sort();
	flags.dedup();
	println!("flags: {:#08X?}", flags);

	Ok(())
}
