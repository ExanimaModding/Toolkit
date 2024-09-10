// Prevents the terminal from opening on a release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod gui;
mod injector;

use std::path::PathBuf;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("{:?}", std::env::current_dir()?);

	gui::start_gui().await?;

	Ok(())
}

pub fn launch_exanima() {
	let exanima_exe = match std::env::var("EXANIMA_EXE") {
		Ok(var) => PathBuf::from(var),
		Err(_) => PathBuf::from("./Exanima.exe"),
	};
	if !exanima_exe.exists() {
		panic!("Could not find Exanima.exe\nEither set EXANIMA_EXE to the full path to Exanima.exe or move EMTK into the game folder")
	}

	unsafe {
		injector::inject(
			r"emf.dll",
			exanima_exe
				.to_str()
				.expect("error while looking for Exanima.exe"),
		)
		.expect("error trying to inject into Exanima.exe");
	}
}
