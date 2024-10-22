pub mod fty;
pub mod rfc;
pub mod rfi;
pub mod rpk;
pub mod wav;

pub use deku;
use deku::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
// re-export pyo3 to allow the emtk-py crate to use pyo3
pub use pyo3;
#[cfg(feature = "python")]
use pyo3::{exceptions::PyIOError, prelude::*, types::PyBytes};

use fty::Fty;
use rfc::Rfc;
use rfi::Rfi;
use rpk::Rpk;
use wav::Wav;

// FIX: Change context to be an enum
// TODO: add contexts for the rest of the file formats
#[derive(Debug, Clone, Default)]
pub struct Context {
	pub rpk: rpk::Context,
	pub size: usize,
}

impl Context {
	pub fn size(self, size: usize) -> Self {
		Self {
			rpk: self.rpk,
			size,
		}
	}
}

#[cfg_attr(feature = "python", pyclass(get_all))]
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(ctx = "ctx: Context", ctx_default = "Context::default()")]
pub struct Unknown {
	#[deku(count = "ctx.size")]
	pub data: Vec<u8>,
}

#[cfg_attr(feature = "python", pyclass)]
// Remove 4 from `size` to account for the magic number
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(
	id_type = "u32",
	ctx = "ctx: Context",
	ctx_default = "Context::default().size(4)"
)]
pub enum Format {
	#[deku(id_pat = "&fty::MAGIC_V1 | &fty::MAGIC_V2")]
	Fty(#[deku(ctx = "ctx.size - 4 ")] Fty),

	#[deku(id = "rfc::MAGIC")]
	Rfc(#[deku(ctx = "ctx.size - 4")] Rfc),

	#[deku(id = "rfi::MAGIC")]
	Rfi(#[deku(ctx = "ctx.size - 4")] Rfi),

	#[deku(id = "rpk::MAGIC")]
	Rpk(#[deku(ctx = "ctx.rpk")] Rpk),

	#[deku(id = "wav::MAGIC")]
	Wav(#[deku(ctx = "ctx.size - 4")] Wav),

	#[deku(id_pat = "_")]
	Unknown(#[deku(ctx = "ctx")] Unknown),
}

#[cfg(feature = "python")]
#[pymethods]
impl Format {
	#[staticmethod]
	#[pyo3(name = "from_bytes")]
	fn from_pybytes(data: &[u8]) -> PyResult<Self> {
		match Format::from_bytes((data, 0)) {
			Ok((_, format)) => Ok(format),
			Err(err) => Err(PyIOError::new_err(err.to_string())),
		}
	}

	#[pyo3(name = "to_bytes")]
	fn to_pybytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
		match self.to_bytes() {
			Ok(buf) => Ok(PyBytes::new_bound(py, &buf)),
			Err(err) => Err(PyIOError::new_err(err.to_string())),
		}
	}
}
