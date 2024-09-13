use deku::prelude::*;
use serde::{Deserialize, Serialize};

use crate::VecReader;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
/// Rayform Database
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[deku(ctx = "size: usize", ctx_default = "0")]
pub struct Rdb {
	#[deku(temp, temp_value = "((entries.len() * 16) + 1) as u32")]
	pub size_of_entries: u32,
	// #[pyo3(get)]
	#[deku(count = "(size_of_entries / 16) - 1")]
	pub entries: Vec<Entry>,
	// #[pyo3(get)]
	#[deku(
		reader = "VecReader::read(deku::reader, size)",
		writer = "VecReader::write(deku::writer, &self.data)"
	)]
	pub data: Vec<u8>,
}

#[cfg(not(feature = "python"))]
/// Rayform Database
#[deku_derive(DekuRead, DekuWrite)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[deku(ctx = "size: usize", ctx_default = "0")]
pub struct Rdb {
	#[deku(temp, temp_value = "((entries.len() * 16) + 1) as u32")]
	pub size_of_entries: u32,
	#[deku(count = "(size_of_entries / 16) - 1")]
	pub entries: Vec<Entry>,
	#[deku(
		reader = "VecReader::read(deku::reader, size)",
		writer = "VecReader::write(deku::writer, &self.data)"
	)]
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
pub struct Entry {
	id: u32,
	flags: Flags,
	offset: u32,
	size: u32,
}

// NOTE: 32 possible flags which would require 32 variants
#[repr(u32)]
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(id_type = "u32")]
enum Flags {
	A = 1 << 0,
	B = 1 << 1,
}
