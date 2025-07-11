pub mod fty;
pub mod rfc;
pub mod rfi;
pub mod rpk;
pub mod wav;

pub use deku;
use deku::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::{exceptions::PyIOError, intern, prelude::*, types::PyBytes};

pub use fty::Fty;
pub use rfc::Rfc;
pub use rfi::Rfi;
pub use rpk::Rpk;
pub use wav::Wav;

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
	Fty {
		id: u32,
		#[deku(ctx = "ctx.size -4")]
		data: Fty,
	},

	#[deku(id = "rfc::MAGIC")]
	Rfc(#[deku(ctx = "ctx.size - 4")] Rfc),

	#[deku(id = "rfi::MAGIC")]
	Rfi(#[deku(ctx = "ctx.size - 4")] Rfi),

	#[deku(id = "rpk::MAGIC")]
	Rpk(#[deku(ctx = "ctx.rpk")] Rpk),

	#[deku(id = "wav::MAGIC")]
	Wav(#[deku(ctx = "ctx.size - 4")] Wav),

	#[deku(id_pat = "_")]
	Unknown {
		id: u32,
		#[deku(ctx = "ctx")]
		data: Unknown,
	},
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

#[cfg(feature = "python")]
#[pymodule(name = "_internal_emtk_asset")]
fn emtk_asset(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
	pyo3_log::init();

	// We need to make the submodules as a package in order to be able to import it in python
	let sys_module = py.import_bound("sys")?.getattr("modules")?;

	// emtk_asset module
	m.add_class::<Unknown>()?;
	m.add_class::<Format>()?;

	// fty module
	let fty = PyModule::new_bound(m.py(), "emtk_asset.fty")?;
	fty.add("MAGIC_V1", fty::MAGIC_V1)?;
	fty.add("MAGIC_V2", fty::MAGIC_V2)?;
	fty.add_class::<Fty>()?;
	m.setattr(intern!(m.py(), "fty"), &fty)?;
	sys_module.set_item("emtk_asset.fty", &fty)?;

	// rfc module
	let rfc = PyModule::new_bound(m.py(), "emtk_asset.rfc")?;
	rfc.add("MAGIC", rfc::MAGIC)?;
	rfc.add_class::<Rfc>()?;
	m.setattr(intern!(m.py(), "rfc"), &rfc)?;
	sys_module.set_item("emtk_asset.rfc", &rfc)?;

	// rfi module
	let rfi = PyModule::new_bound(m.py(), "emtk_asset.rfi")?;
	rfi.add("MAGIC", rfi::MAGIC)?;
	rfi.add_class::<Rfi>()?;
	m.setattr(intern!(m.py(), "rfi"), &rfi)?;
	sys_module.set_item("emtk_asset.rfi", &rfi)?;

	// rpk module
	let rpk = PyModule::new_bound(m.py(), "emtk_asset.rpk")?;
	rpk.add("MAGIC", rpk::MAGIC)?;
	rpk.add_class::<rpk::Entry>()?;
	rpk.add_class::<Rpk>()?;
	m.setattr(intern!(m.py(), "rpk"), &rpk)?;
	sys_module.set_item("emtk_asset.rpk", &rpk)?;

	// wav module
	let wav = PyModule::new_bound(m.py(), "emtk_asset.wav")?;
	wav.add("MAGIC", wav::MAGIC)?;
	wav.add_class::<Wav>()?;
	m.setattr(intern!(m.py(), "wav"), &wav)?;
	sys_module.set_item("emtk_asset.wav", &wav)?;

	Ok(())
}
