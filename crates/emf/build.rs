use std::env;
use std::fs::read_dir;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Export DLL Ordinals
	let mut ordinal = project_root().unwrap();
	ordinal.push("crates");
	ordinal.push("emf");
	ordinal.push("lib.def");
	println!("cargo:rustc-cdylib-link-arg=/DEF:{}", ordinal.display());

	Ok(())
}

fn project_root() -> io::Result<PathBuf> {
	let path = env::current_dir()?;
	let path_ancestors = path.as_path().ancestors();

	for p in path_ancestors {
		let has_cargo = read_dir(p)?.any(|p| p.unwrap().file_name() == *"Cargo.lock");
		if has_cargo {
			return Ok(PathBuf::from(p));
		}
	}
	Err(io::Error::new(
		ErrorKind::NotFound,
		"Ran out of places to find Cargo.toml",
	))
}
