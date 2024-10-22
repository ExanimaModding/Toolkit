use deku::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub const MAGIC: u32 = 0x1D2D3DC6;

#[cfg_attr(feature = "python", pyclass(get_all))]
/// Rayform Image
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(ctx = "size: usize", ctx_default = "0")]
pub struct Rfi {
	#[deku(count = "size")]
	pub data: Vec<u8>,
}
