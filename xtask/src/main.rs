use std::{
	env, fs,
	path::{Path, PathBuf},
	process,
};

/// # Examples
///
/// `cargo xtask run`
///
/// `cargo xtask` to print all commands to the stdout
fn main() {
	let task = env::args().nth(1);
	match task {
		None => print_help(),
		Some(t) => match t.as_str() {
			"run" => run(),
			_ => run_plugin(&t, None),
		},
	}
}

/// Parses the examples folder to print out to the stdout as a xtask command
fn print_help() {
	let mut descriptions: Vec<String> = Vec::new();

	// Format other commands here
	descriptions.push("Tasks:\n".to_string());
	descriptions.push("run                Run all example plugins\n".to_string());

	let project_root = project_root();
	let examples_path = project_root.join("examples");
	for entry in examples_path
		.read_dir()
		.expect("error while reading examples folder")
		.flatten()
	{
		let path = entry.path();
		if path.is_file() {
			continue;
		}
		let name = entry
			.file_name()
			.to_str()
			.expect("error while getting name of entry")
			.to_string();

		descriptions.push(format!(
			"{}            Run only the {} plugin\n",
			name, name
		));
	}

	eprintln!(
		"{}",
		descriptions
			.iter()
			.flat_map(|s| s.chars())
			.collect::<String>()
	);
}

fn cargo_env() -> String {
	env::var("CARGO").unwrap_or("cargo".to_string())
}

fn project_root() -> &'static Path {
	Path::new(&env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

/// Reads the EXANIMA_EXE environment variable and returns it as a [`PathBuf`].
///
/// A default install of Steam will install Exanima at this path:
///
/// ```rust
/// "C:/Program Files (x86)/Steam/steamapps/common/Exanima/Exanima.exe"
/// ```
///
/// # Panics
///
/// EXANIMA_EXE environment variable **must** be set.
///
/// EXANIMA_EXE **must** point to an existing file. The file should be the game's binary.
fn exe_path() -> PathBuf {
	let exanima_exe = PathBuf::from(
		env::var("EXANIMA_EXE").expect("environment variable, EXANIMA_EXE, must be set"),
	);
	if !exanima_exe.exists() {
		panic!("Could not find Exanima.exe\nSet EXANIMA_EXE to the full path to Exanima.exe")
	}

	exanima_exe
}

/// Run all example plugins
fn run() {
	let cargo = cargo_env();
	let project_root = project_root();
	let examples_path = project_root.join("examples");
	let exe_path = exe_path();

	for entry in examples_path
		.read_dir()
		.expect("error while reading examples folder")
		.flatten()
	{
		let path = entry.path();
		if path.is_file() {
			continue;
		}
		let name = entry
			.file_name()
			.to_str()
			.expect("error while getting name of entry")
			.to_string();

		run_plugin(&name, Some(exe_path.clone()));
	}

	process::Command::new(cargo)
		.current_dir(project_root)
		.args(["run"])
		.status()
		.expect(r#"error while running "cargo run""#);
}

/// Run only one plugin by name
fn run_plugin(name: &str, exanima_exe_path: Option<PathBuf>) {
	let cargo = cargo_env();
	let project_root = project_root();
	let example_path = project_root.join(format!("examples/{}", name));
	if !example_path.exists() {
		eprintln!("\"{}\" is an invalid command\n", name);
		print_help();
		return;
	}
	let exe_path = match exanima_exe_path.clone() {
		Some(path) => path,
		None => exe_path(),
	};
	let build_path = project_root.join("target/debug");
	let plugin_path = exe_path.parent().unwrap().join(format!("mods/{}", name));

	process::Command::new(cargo.clone())
		.current_dir(project_root)
		.args(["build", "-p", name])
		.status()
		.unwrap_or_else(|e| panic!("error while building {}: {}", name, e));

	fs::create_dir_all(plugin_path.clone())
		.unwrap_or_else(|e| panic!("error while creating {} folder at mods path: {}", name, e));
	fs::copy(
		build_path.join(format!("{}.dll", name)),
		plugin_path.join(format!("{}.dll", name)),
	)
	.unwrap_or_else(|e| panic!("error while copying {} dll to mods folder: {}", name, e));
	// Do not overwrite config if it exists
	if !plugin_path.join("config.toml").exists() {
		fs::copy(
			example_path.join("config.toml"),
			plugin_path.join("config.toml"),
		)
		.unwrap_or_else(|e| panic!("error while copying {} config to mods folder: {}", name, e));
	}

	// Skip when using "cargo xtask run"
	if exanima_exe_path.is_none() {
		process::Command::new(cargo)
			.current_dir(project_root)
			.args(["run"])
			.status()
			.unwrap();
	}
}
