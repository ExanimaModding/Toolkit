use exparser::{
	fty::{self, Fty},
	pyo3::{intern, prelude::*},
	rfc::{self, Rfc},
	rfi::{self, Rfi},
	rpk::{self, Rpk},
	wav::{self, Wav},
	Format, Unknown,
};

#[pymodule(name = "_internal_emtk_py", crate = "exparser::pyo3")]
fn emtk_py(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
	pyo3_log::init();

	// We need to make the submodules as a package in order to be able to import it in python
	let sys_module = py.import_bound("sys")?.getattr("modules")?;

	// exparser module
	let exparser = PyModule::new_bound(m.py(), "emtk.exparser")?;
	exparser.add_class::<Unknown>()?;
	exparser.add_class::<Format>()?;
	m.setattr(intern!(m.py(), "exparser"), &exparser)?;
	sys_module.set_item("emtk.exparser", &exparser)?;

	// fty module
	let fty = PyModule::new_bound(exparser.py(), "emtk.exparser.fty")?;
	fty.add("MAGIC_V1", fty::MAGIC_V1)?;
	fty.add("MAGIC_V2", fty::MAGIC_V2)?;
	fty.add_class::<Fty>()?;
	exparser.setattr(intern!(exparser.py(), "fty"), &fty)?;
	sys_module.set_item("emtk.exparser.fty", &fty)?;

	// rfc module
	let rfc = PyModule::new_bound(exparser.py(), "emtk.exparser.rfc")?;
	rfc.add("MAGIC", rfc::MAGIC)?;
	rfc.add_class::<Rfc>()?;
	exparser.setattr(intern!(exparser.py(), "rfc"), &rfc)?;
	sys_module.set_item("emtk.exparser.rfc", &rfc)?;

	// rfi module
	let rfi = PyModule::new_bound(exparser.py(), "emtk.exparser.rfi")?;
	rfi.add("MAGIC", rfi::MAGIC)?;
	rfi.add_class::<Rfi>()?;
	exparser.setattr(intern!(exparser.py(), "rfi"), &rfi)?;
	sys_module.set_item("emtk.exparser.rfi", &rfi)?;

	// rpk module
	let rpk = PyModule::new_bound(exparser.py(), "emtk.exparser.rpk")?;
	rpk.add("MAGIC", rpk::MAGIC)?;
	rpk.add_class::<rpk::Entry>()?;
	rpk.add_class::<Rpk>()?;
	exparser.setattr(intern!(exparser.py(), "rpk"), &rpk)?;
	sys_module.set_item("emtk.exparser.rpk", &rpk)?;

	// wav module
	let wav = PyModule::new_bound(exparser.py(), "emtk.exparser.wav")?;
	wav.add("MAGIC", wav::MAGIC)?;
	wav.add_class::<Wav>()?;
	exparser.setattr(intern!(exparser.py(), "wav"), &wav)?;
	sys_module.set_item("emtk.exparser.wav", &wav)?;

	Ok(())
}
