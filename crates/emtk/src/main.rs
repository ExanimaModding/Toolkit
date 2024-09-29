mod injector;

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let exanima_exe = match std::env::var("EXANIMA_EXE") {
		Ok(var) => PathBuf::from(var),
		Err(_) => PathBuf::from("./Exanima.exe"),
	};
	if !exanima_exe.exists() {
		panic!("Could not find Exanima.exe\nEither set EXANIMA_EXE to the full path to Exanima.exe or move EMTK into the game folder")
	}

	let ld_library_path = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();

	#[cfg(debug_assertions)]
	let emf_dll = {
		let path = ld_library_path
			.split(':')
			.map(|dir| PathBuf::from(dir).join("emf.dll"))
			.find(|path| path.exists())
			.unwrap_or_else(|| {
				panic!("Could not find emf.dll in any of the directories in LD_LIBRARY_PATH")
			});
		let path = path.canonicalize().unwrap();
		let path = path.to_string_lossy();
		path.to_string()
	};

	#[cfg(not(debug_assertions))]
	let emf_dll = "emf.dll".to_string();

	unsafe {
		injector::inject(
			&emf_dll,
			exanima_exe
				.to_str()
				.expect("error while looking for Exanima.exe"),
		)
		.expect("error trying to inject into Exanima.exe");
	}
	Ok(())
}
