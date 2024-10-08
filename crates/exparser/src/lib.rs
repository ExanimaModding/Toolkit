pub mod fty;
pub mod rfc;
pub mod rfi;
pub mod rpk;
pub mod wav;

use std::io;

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
	#[deku(
		reader = "VecReader::read(deku::reader, ctx.size)",
		writer = "VecReader::write(deku::writer, &self.data)"
	)]
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

// TODO: rename struct
pub(crate) struct VecReader;

impl VecReader {
	pub(crate) fn read<R: io::Read + io::Seek>(
		reader: &mut Reader<R>,
		size: usize,
	) -> Result<Vec<u8>, DekuError> {
		let mut buf = vec![0; size];
		reader.read_bytes(size, &mut buf)?;
		Ok(buf)
	}

	pub(crate) fn write<W: io::Write + io::Seek>(
		writer: &mut Writer<W>,
		data: &Vec<u8>,
	) -> Result<(), DekuError> {
		writer.write_bytes(data.as_slice())
	}

	pub(crate) fn write_nested<W: io::Write + io::Seek>(
		writer: &mut Writer<W>,
		data: &Vec<Vec<u8>>,
	) -> Result<(), DekuError> {
		for vec in data {
			Self::write(writer, vec)?;
		}
		Ok(())
	}
}
