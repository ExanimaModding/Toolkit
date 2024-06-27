pub mod injector;

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let exanima_path = match std::env::var("EXANIMA_PATH") {
		Ok(var) => PathBuf::from(var),
		Err(_) => PathBuf::from("./Exanima.exe"),
	};
	if !exanima_path.exists() {
		panic!("Could not find Exanima.exe\nEither set EXANIMA_PATH to the Exanima exe or move EMTK into the game folder")
	}

	unsafe {
		injector::inject(
			r"emf.dll",
			exanima_path
				.to_str()
				.expect("error while looking for exanima"),
		)
		.unwrap();
	}
	Ok(())
}
