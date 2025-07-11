use deku::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub const MAGIC_V1: u32 = 0xAFCE0F00;
pub const MAGIC_V2: u32 = 0xAFCE0F01;

#[cfg_attr(feature = "python", pyclass(get_all))]
/// Factory
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(ctx = "size: usize", ctx_default = "0")]
pub struct Fty {
	#[deku(count = "size")]
	pub data: Vec<u8>,
}
