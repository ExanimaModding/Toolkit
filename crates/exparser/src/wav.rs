use crate::VecReader;
use deku::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub const MAGIC: u32 = 0x46464952;

#[cfg_attr(feature = "python", pyclass(get_all))]
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(ctx = "size: usize", ctx_default = "0")]
pub struct Wav {
	#[deku(
		reader = "VecReader::read(deku::reader, size)",
		writer = "VecReader::write(deku::writer, &self.data)"
	)]
	pub data: Vec<u8>,
}
