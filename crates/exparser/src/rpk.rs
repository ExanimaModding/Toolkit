use std::io;

use deku::{prelude::*, reader::ReaderRet};
use log::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub const MAGIC: u32 = 0xAFBF0C01;

#[derive(Debug, Clone, Default)]
pub struct Context {
	/// Whether to skip the `Rpk` data field or not.
	///
	/// By default, the data field will be read by the reader. If you want to save memory and have
	/// better performance, and the data field is not needed such as when simply viewing a list of
	/// entries, setting this to `true` may be desired.
	pub entries_only: bool,
	/// Filter the `Rpk` data field by a desired `Vec<Entry>`.
	///
	/// By default, no filter is applied thus all of the data is read by the reader. If only data
	/// from certain entries is desired, provide this field with your desired entries.
	pub entries: Option<Vec<Entry>>,
}

#[cfg(feature = "python")]
/// Rayform Package
#[pyclass]
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[deku(ctx = "ctx: Context", ctx_default = "Context::default()")]
pub struct Rpk {
	#[deku(temp, temp_value = "(entries.len() * 32) as u32")]
	pub size_of_entries: u32,
	#[pyo3(get)]
	#[deku(count = "size_of_entries / 32")]
	pub entries: Vec<Entry>,
	#[pyo3(get)]
	#[deku(
		cond = "!ctx.entries_only",
		reader = "Rpk::read(ctx, deku::reader, entries)",
		writer = "Rpk::write(deku::writer, &self.data)"
	)]
	pub data: Vec<Vec<u8>>,
}

#[cfg(not(feature = "python"))]
/// Rayform Package
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[deku(ctx = "ctx: Context", ctx_default = "Context::default()")]
pub struct Rpk {
	#[deku(temp, temp_value = "(entries.len() * 32) as u32")]
	pub size_of_entries: u32,
	#[deku(count = "size_of_entries / 32")]
	pub entries: Vec<Entry>,
	#[deku(
		cond = "!ctx.entries_only",
		reader = "Rpk::read(ctx, deku::reader, entries)",
		writer = "Rpk::write(deku::writer, &self.data)"
	)]
	pub data: Vec<Vec<u8>>,
}

impl Rpk {
	fn read<R: io::Read + io::Seek>(
		ctx: Context,
		reader: &mut Reader<R>,
		entries: &[Entry],
	) -> Result<Vec<Vec<u8>>, DekuError> {
		// Sort the entries by offset so we can read them in order.
		let mut entries = match ctx.entries {
			Some(entries) => entries,
			None => entries.to_vec(),
		};
		entries.sort_by(|a, b| a.offset.cmp(&b.offset));

		let mut formats: Vec<Vec<u8>> = Vec::with_capacity(entries.len());

		for entry in entries {
			let mut buf = vec![0; entry.size as usize];

			// deku is slow at reading Vec<u8> so we read the bytes into a buffer ourselves.
			let format = match reader.read_bytes(entry.size as usize, &mut buf) {
				Ok(ReaderRet::Bytes) => Ok(buf),
				Ok(ReaderRet::Bits(_)) => {
					Err(DekuError::InvalidParam("Expected bytes, got bits".into()))
				}
				Err(err) => Err(err),
			}?;

			formats.push(format);
		}
		Ok(formats)
	}

	fn write<W: io::Write + io::Seek>(
		writer: &mut Writer<W>,
		data: &Vec<Vec<u8>>,
	) -> Result<(), DekuError> {
		for vec in data {
			writer.write_bytes(vec.as_slice())?;
		}
		Ok(())
	}
}

#[cfg_attr(feature = "python", pyclass(get_all))]
#[derive(Debug, DekuRead, DekuWrite, Deserialize, Serialize, Clone)]
pub struct Entry {
	#[deku(
		reader = "Entry::read(deku::reader)",
		writer = "Entry::write(deku::writer, &self.name)"
	)]
	pub name: String,
	pub offset: u32,
	#[deku(pad_bytes_after = "8")]
	pub size: u32,
}

impl Entry {
	fn read<R: io::Read + io::Seek>(reader: &mut Reader<R>) -> Result<String, DekuError> {
		let mut buf: Vec<u8> = vec![0; 16];
		match reader.read_bytes(16, &mut buf) {
			Ok(ReaderRet::Bytes) => {
				let mut s = String::new();
				for b in buf {
					match b {
						0 => break,
						v => s.push(v as char),
					}
				}
				Ok(s)
			}
			Ok(ReaderRet::Bits(_)) => {
				Err(DekuError::InvalidParam("Expected bytes, got bits".into()))
			}
			Err(err) => Err(err),
		}
	}

	fn write<W: io::Write + io::Seek>(
		writer: &mut Writer<W>,
		value: &str,
	) -> Result<(), DekuError> {
		let mut buf = vec![0; 16];
		for (i, c) in value.chars().enumerate() {
			if i >= 16 {
				warn!(
					r#"entry name "{}" has been truncated due to being longer than 16 characters."#,
					value
				);
				break;
			}
			buf[i] = c as u8;
		}
		writer.write_bytes(&buf)
	}
}
