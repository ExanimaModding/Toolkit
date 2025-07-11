use deku::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub const MAGIC: u32 = 0x1D2D3DC6;

#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
#[derive(
	Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, DekuRead, DekuWrite, Deserialize, Serialize,
)]
#[deku(id_type = "u32")]
pub enum Unknown1 {
	// TODO: determine the names for all unknowns
	#[deku(id = 0x00000000)]
	Unknown1,
	#[deku(id = 0x00000001)]
	Unknown2,
}

#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
#[derive(
	Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, DekuRead, DekuWrite, Deserialize, Serialize,
)]
#[deku(id_type = "u32")]
pub enum FourCC {
	// TODO: determine the names for all unknowns
	#[deku(id = 0x01002008)]
	Unknown1,
	#[deku(id = 0x01004200)]
	Unknown2,
	#[deku(id = 0x01006208)]
	Unknown3,
	#[deku(id = 0x0100C600)]
	Unknown4,
	#[deku(id = 0x0100E608)]
	Unknown5,
	#[deku(id = 0x01114200)]
	Unknown6,
	#[deku(id = 0x0111C600)]
	Unknown7,
	#[deku(id = 0x11118400)]
	Unknown8,
	#[deku(id = 0x1111C600)]
	Unknown9,
	#[deku(id = 0x813B4200)]
	Unknown10,
	#[deku(id = 0x813BC600)]
	Unknown11,
	#[deku(id = 0x817BE608)]
	Unknown12,
	#[deku(id = 0x823BC600)]
	Unknown13,
	#[deku(id = 0x827BA408)]
	Unknown14,
	#[deku(id = 0x927B8400)]
	Unknown15,
}

#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
#[derive(
	Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, DekuRead, DekuWrite, Deserialize, Serialize,
)]
#[deku(id_type = "u32")]
pub enum Flags {
	#[deku(id = 0x10000000)]
	Uncompressed,
	// Run-length encoding
	#[deku(id = 0x50000000)]
	Compressed,
}

#[cfg_attr(feature = "python", pyclass(get_all))]
/// Rayform Image
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(ctx = "size: usize", ctx_default = "0")]
pub struct Rfi {
	pub width: u32,
	pub height: u32,
	pub _unknown1: Unknown1,
	pub four_cc: FourCC,
	pub _unknown2: u32,
	pub flags: Flags,
	// TODO: replace ctx.size with this field
	pub _size: u32,
	#[deku(count = "size")]
	pub data: Vec<u8>,
}
