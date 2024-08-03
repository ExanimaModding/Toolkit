pub mod fty;
pub mod rfc;
pub mod rfi;
pub mod rpk;
pub mod wav;

use std::io;

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

#[cfg_attr(feature = "python", pyclass(get_all))]
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(ctx = "size: usize", ctx_default = "0")]
pub struct Unknown {
	#[deku(
		reader = "VecReader::read(deku::reader, size)",
		writer = "VecReader::write(deku::writer, &self.data)"
	)]
	pub data: Vec<u8>,
}

#[cfg_attr(feature = "python", pyclass)]
// Remove 4 from `size` to account for the magic number
#[derive(Clone, Debug, DekuRead, DekuWrite, Deserialize, Serialize)]
#[deku(id_type = "u32", ctx = "size: usize", ctx_default = "4")]
pub enum Format {
	#[deku(id_pat = "&fty::MAGIC_V1 | &fty::MAGIC_V2")]
	Fty(#[deku(ctx = "size -4 ")] Fty),

	#[deku(id = "rfc::MAGIC")]
	Rfc(#[deku(ctx = "size - 4")] Rfc),

	#[deku(id = "rfi::MAGIC")]
	Rfi(#[deku(ctx = "size- 4")] Rfi),

	#[deku(id = "rpk::MAGIC")]
	Rpk(Rpk),

	#[deku(id = "wav::MAGIC")]
	Wav(#[deku(ctx = "size - 4")] Wav),

	#[deku(id_pat = "_")]
	Unknown(#[deku(ctx = "size- 4")] Unknown),
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
}
