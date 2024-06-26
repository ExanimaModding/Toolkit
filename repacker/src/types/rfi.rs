use crate::{
	metadata::MagicBytes,
	types::dds,
	utils::{any_as_u8_slice, red, ReadSeek, SourceData},
};
use bitstream_io::{
	read::{BitRead, BitReader},
	write::{BitWrite, BitWriter},
	LittleEndian,
};
use std::{
	fs::{create_dir_all, File},
	io::{Cursor, SeekFrom},
	path::PathBuf,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("RLE is unsupported")]
	UnsupportedRLE(String),
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum S3TC {
	DXT1(u32),
	DXT5(u32),
	BC4U(u32),
	BC5U(u32),
	RGB(u32),
}

impl TryFrom<u32> for S3TC {
	type Error = &'static str;

	fn try_from(n: u32) -> std::result::Result<Self, Self::Error> {
		match n {
			0x813BC600 => Ok(S3TC::DXT1(0x813BC600)),
			0x823BC600 => Ok(S3TC::DXT1(0x823BC600)),
			0x01004200 => Ok(S3TC::DXT1(0x01004200)),
			0x817BE608 => Ok(S3TC::DXT5(0x817BE608)),
			0x0111C600 => Ok(S3TC::BC4U(0x0111C600)),
			0x01006208 => Ok(S3TC::BC4U(0x01006208)),
			0x813B4200 => Ok(S3TC::BC4U(0x813B4200)),
			0x01114200 => Ok(S3TC::BC4U(0x01114200)),
			0x01002008 => Ok(S3TC::BC4U(0x01002008)),
			0x0100E608 => Ok(S3TC::BC4U(0x0100E608)),
			0x11118400 => Ok(S3TC::BC4U(0x11118400)),
			0x927B8400 => Ok(S3TC::BC5U(0x927B8400)),
			0x827BA408 => Ok(S3TC::BC5U(0x827BA408)),
			0x1111C600 => Ok(S3TC::BC5U(0x1111C600)),
			0x827BE608 => Ok(S3TC::BC5U(0x827BE608)),
			0x0100C600 => Ok(S3TC::RGB(0x0100C600)),
			_ => Err("Cannot get S3TC from number"),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Header {
	pub width: u32,
	pub height: u32,
	pub _unknown1: u32,
	pub s3tc: u32,
	pub _unknown2: u32,
	pub flags: u32,
	pub size: u32,
}

impl Header {
	pub fn new(src: SourceData) -> Result<Self, Box<dyn std::error::Error>> {
		let path_str = match src.clone() {
			SourceData::Path(path) => path,
			SourceData::Buffer(path, _) => path,
		};
		let path = PathBuf::from(path_str);

		let buffer = match src {
			SourceData::Path(path) => Box::new(File::open(path)?) as Box<dyn ReadSeek>,
			SourceData::Buffer(_, buf) => Box::new(Cursor::new(buf)) as Box<dyn ReadSeek>,
		};

		let mut reader = BitReader::endian(buffer, LittleEndian);

		let magic = reader.read::<u32>(32)?;
		let magic = MagicBytes::try_from(magic)?;
		if magic != MagicBytes::RFI {
			panic!(
				"❗ '{}' must be a RFI format",
				red(path.file_name().unwrap().to_str().unwrap())
			);
		}

		let header = Header {
			width: reader.read::<u32>(32)?,
			height: reader.read::<u32>(32)?,
			_unknown1: reader.read::<u32>(32)?,
			s3tc: reader.read::<u32>(32)?,
			_unknown2: reader.read::<u32>(32)?,
			flags: reader.read::<u32>(32)?,
			size: reader.read::<u32>(32)?,
		};

		Ok(header)
	}
}

#[derive(Debug)]
pub struct RFI {
	pub header: Header,
}

impl RFI {
	pub fn new(path: SourceData) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(RFI {
			header: Header::new(path)?,
		})
	}

	pub fn unpack(&self, src: SourceData, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
		let is_rle = self.header.flags & 0x40000000 == 0x40000000;
		if is_rle {
			return Err(Box::new(Error::UnsupportedRLE(String::from(
				"RLE not implemented yet",
			))));
		}

		let src_str = match src.clone() {
			SourceData::Path(path) => path,
			SourceData::Buffer(path, _) => path,
		};
		let src_path = PathBuf::from(src_str);
		let mut dest_path = PathBuf::from(dest);

		let dds = dds::Header::new(self)?;

		create_dir_all(&dest_path)?;
		dest_path.push(src_path.file_stem().unwrap().to_str().unwrap());
		dest_path.set_extension("dds");

		let mut writer = BitWriter::endian(File::create(&dest_path)?, LittleEndian);

		// DDS magic
		writer.write_bytes(b"DDS ")?;

		unsafe {
			let header = any_as_u8_slice(&dds);
			writer.write_bytes(header)?;
		}

		let buffer = match src {
			SourceData::Path(path) => Box::new(File::open(path)?) as Box<dyn ReadSeek>,
			SourceData::Buffer(_, buf) => Box::new(Cursor::new(buf)) as Box<dyn ReadSeek>,
		};

		let mut reader = BitReader::endian(buffer, LittleEndian);

		// Seek past RFI header
		reader.seek_bits(SeekFrom::Start(32 * 8))?;

		let data = reader.read_to_vec((self.header.size) as usize)?;
		writer.write_bytes(data.as_slice())?;

		Ok(())
	}
}
