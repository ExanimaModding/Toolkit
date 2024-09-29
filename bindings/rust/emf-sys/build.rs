//! We download the latest release of emf.dll.lib from the repository and save it to target/<profile>/deps/emf.dll.lib.
//!
//! This is so that users don't have to build the entirety of EMF to write plugins.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Release {
	tag_name: String,
	assets: Vec<Asset>,
}

static LATEST_RELEASE: &str =
	"https://codeberg.org/api/v1/repos/ExanimaModding/Toolkit/releases/latest";

#[derive(Serialize, Deserialize)]
struct Asset {
	id: usize,
	name: String,
	size: usize,
	download_count: usize,
	created_at: String,
	uuid: String,
	browser_download_url: String,
}

pub fn main() {
	let out_dir = std::env::var("OUT_DIR").unwrap();
	let mut out_file: PathBuf = out_dir.into();
	out_file.pop();
	out_file.pop();
	out_file.pop();
	out_file.push("deps");

	println!("cargo:rustc-link-search={}", out_file.to_str().unwrap());

	out_file.push("emf.dll.lib");

	let metadata = std::fs::metadata(&out_file);

	// Only download the latest release if the file doesn't exist.
	if metadata.is_ok() {
		return;
	}

	let result = ureq::get(LATEST_RELEASE).call();

	if let Ok(result) = result {
		let release: Release = result.into_json().unwrap();

		let asset = release
			.assets
			.iter()
			.find(|asset| asset.name.ends_with(".zip"));

		if let Some(asset) = asset {
			let url = &asset.browser_download_url;
			let mut bytes: Vec<u8> = Vec::with_capacity(asset.size);
			let read = ureq::get(url)
				.call()
				.unwrap()
				.into_reader()
				.read_to_end(&mut bytes)
				.unwrap();
			if read != asset.size {
				panic!("Failed to download the file. Incorrect size.");
			}

			let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
			let mut emf_lib = zip.by_name("emf.dll.lib").unwrap();

			let mut lib = std::fs::File::create(out_file).unwrap();
			std::io::copy(&mut emf_lib, &mut lib).unwrap();
		} else {
			panic!("Failed to find the asset.");
		}
	} else {
		panic!("Failed to get the latest release of emf.dll.lib");
	}
}
