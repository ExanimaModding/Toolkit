use std::env;
use std::fs::{File, read_dir};
use std::io::{self, ErrorKind};
use std::path::PathBuf;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Export DLL Ordinals
	let mut ordinal = project_root().unwrap();
	ordinal.push("emtk_framework");
	ordinal.push("lib.def");
	println!("cargo:rustc-cdylib-link-arg=/DEF:{}", ordinal.display());

	download_libmem()?;

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

fn download_libmem() -> Result<(), Box<dyn std::error::Error>> {
	use flate2::read::GzDecoder;
	use tar::Archive;

	// Taken from libmem's github repo, modified to download the static MD builds, to make dependencies happy.
	let version = "5.0.4";
	let os_name = env::var("CARGO_CFG_TARGET_OS").unwrap();
	let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
	let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();

	let archive_name = format!(
		"libmem-{}-{}-{}-{}-static-md",
		version, arch, os_name, target_env
	);

	let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
	let archive_path = out_dir.join(format!("{}.tar.gz", archive_name));

	let download_url =
		format!("https://github.com/rdbo/libmem/releases/download/{version}/{archive_name}.tar.gz");

	println!("Downloading {}", download_url);

	if !archive_path.exists() {
		let mut req = reqwest::blocking::get(&download_url)?;
		let mut file = std::fs::File::create(&archive_path)?;
		std::io::copy(&mut req, &mut file)?;
	}

	let archive_dir = out_dir.join(archive_name);
	if !archive_dir.exists() {
		let tar_gz = File::open(&archive_path)?;
		let tar = GzDecoder::new(tar_gz);
		let mut tar_archive = Archive::new(tar);
		tar_archive.unpack(out_dir)?;
	}

	let search_path = archive_dir.join("lib").join("release");
	println!("cargo:rustc-link-search={}", search_path.display());
	Ok(())
}
