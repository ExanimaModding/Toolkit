use std::{
	env, fs, io,
	path::{Path, PathBuf},
	process,
};

#[derive(PartialEq)]
enum BuildMode {
	Release,
	Dev,
}

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
			"blender" => match env::args().nth(2) {
				None => blender(BuildMode::Dev),
				Some(t) => match t.as_str() {
					"-r" => blender(BuildMode::Release),
					"-h" => blender_help(),
					_ => blender(BuildMode::Dev),
				},
			},
			"python" => match env::args().nth(2) {
				None => python(BuildMode::Dev),
				Some(t) => match t.as_str() {
					"-r" => python(BuildMode::Release),
					"-h" => python_help(),
					_ => python(BuildMode::Dev),
				},
			},
			"run" => run(),
			"wheel" => match env::args().nth(2) {
				None => wheel(BuildMode::Dev),
				Some(t) => match t.as_str() {
					"-r" => wheel(BuildMode::Release),
					"-h" => wheel_help(),
					_ => wheel(BuildMode::Dev),
				},
			},
			_ => run_plugin(&t, None),
		},
	}
}

fn blender_help() {
	let descriptions = &[
		"Blender task flags:\n",
		"-r                 Build the blender extension with blender's CLI for distribution\n",
		"-h                 Shows this message\n",
	];
	eprintln!("{}", descriptions.join(""))
}

fn python_help() {
	let descriptions = &[
		"Python task flags:\n",
		"-r                 Develop with the emtk-py python bindings using maturin in release mode\n",
		"-h                 Shows this message\n",
	];
	eprintln!("{}", descriptions.join(""))
}

fn wheel_help() {
	let descriptions = &[
		"Wheel task flags:\n",
		"-r                 Build emtk-py wheel with maturin in release mode\n",
		"-h                 Shows this message\n",
	];
	eprintln!("{}", descriptions.join(""))
}

/// Parses the examples folder to print out to the stdout as a xtask command
fn print_help() {
	let mut descriptions = vec![
		"Tasks:\n".to_string(),
		"blender            Copy emtk-py's blender folder to blender's extension folder (-h for help)\n".to_string(),
		"python             Develop with emtk-py python bindings using maturin (-h for help)\n".to_string(),
		"run                Run all example plugins\n".to_string(),
		"wheel              Build emtk-py wheel with maturin (-h for help)\n".to_string(),
	];

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
			"{}{}Run only the {} plugin\n",
			name,
			" ".repeat(19 - name.len()),
			name
		));
	}

	eprintln!("{}", descriptions.join(""));
}

/// Helps show exactly what command ran with what arguments in the panic
fn panic_command(cmd: &str, args: Option<&[&str]>, e: io::Error) -> process::ExitStatus {
	match args {
		None => {
			panic!(r#"error while running "{}": {}"#, cmd, e)
		}
		Some(a) => {
			panic!(r#"error while running "{} {}": {}"#, cmd, a.join(" "), e)
		}
	}
}

/// This will usually return "cargo"
fn cargo_env() -> String {
	env::var("CARGO").unwrap_or("cargo".to_string())
}

fn project_root() -> &'static Path {
	Path::new(&env!("CARGO_MANIFEST_DIR"))
		.parent()
		.unwrap()
		.parent()
		.unwrap()
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
/// - EXANIMA_EXE environment variable **must** be set.
/// - EXANIMA_EXE **must** point to an existing file. The file should be the game's binary.
fn exe_path() -> PathBuf {
	let exanima_exe = PathBuf::from(
		env::var("EXANIMA_EXE").expect("environment variable, EXANIMA_EXE, must be set"),
	);
	if !exanima_exe.exists() {
		panic!("Could not find Exanima.exe\nSet EXANIMA_EXE to the full path to Exanima.exe")
	}

	exanima_exe
}

fn setup_python() {
	let project_root = project_root();
	let uv_cmd = "uv";
	let uv_venv_args = &["venv"];
	let uv_install_args = &[
		"pip",
		"install",
		"-r",
		"./bindings/python/emtk-py/requirements.txt",
	];

	process::Command::new(uv_cmd)
		.current_dir(project_root)
		.args(uv_venv_args)
		.status()
		.unwrap_or_else(|e| panic_command(uv_cmd, Some(uv_venv_args), e));
	process::Command::new(uv_cmd)
		.current_dir(project_root)
		.args(uv_install_args)
		.status()
		.unwrap_or_else(|e| panic_command(uv_cmd, Some(uv_install_args), e));
}

/// A zip file will be created when building for release.
/// When building in dev mode, emtk-py's blender folder is copied to blender's extension folder.
///
/// Blender 4.2 supported only
///
/// # Panics
///
/// - The uv command **must** be in the PATH environment variable
/// - The blender binary **must** be in the PATH environment variable when building in release
fn blender(build_mode: BuildMode) {
	let unsupported_platform = "This task is currently unsupported on this system";
	let project_root = project_root();
	// NOTE: emtk-py wheel
	let wheel_pkg = if cfg!(windows) {
		"wheels/emtk-0.1.0b1-cp311-abi3-win_amd64.whl"
	} else if cfg!(unix) {
		"wheels/emtk-0.1.0b1-cp311-abi3-manylinux_2_34_x86_64.whl"
	} else {
		return eprintln!("{}", unsupported_platform);
	};

	// Copy the emtk-py wheel file into emtk's blender extension "wheels" folder
	let bl_dep_path = PathBuf::from(&wheel_pkg);
	let wheel_name = bl_dep_path.file_name().unwrap().to_str().unwrap();
	let wheel_path = project_root.join(format!("target/wheels/{}", &wheel_name));
	let target_wheel_path = project_root.join(format!(
		"bindings/python/emtk-py/emtk/blender/wheels/{}",
		&wheel_name
	));
	let target_wheel_parent = target_wheel_path.parent().unwrap();
	if !target_wheel_parent.exists() {
		fs::create_dir_all(target_wheel_parent).unwrap();
	}
	if !wheel_path.exists() {
		wheel(BuildMode::Release);
	}
	fs::copy(&wheel_path, &target_wheel_path).unwrap();

	if build_mode == BuildMode::Release {
		// TODO: Add cross-compilation support
		todo!("Add cross-compilation support");

		// Bundle extension for distribution
		let blender_cmd = "blender";
		let blender_args = &[
			"--command",
			"extension",
			"build",
			"--source-dir",
			"./bindings/python/emtk-py/emtk/blender/",
			"--output-dir",
			"./target/",
		];
		process::Command::new(blender_cmd)
			.current_dir(project_root)
			.args(blender_args)
			.status()
			.unwrap_or_else(|e| panic_command(blender_cmd, Some(blender_args), e));
	} else if build_mode == BuildMode::Dev {
		// Check if the extension folder exists first
		let mut data_dir = if cfg!(windows) {
			PathBuf::from(env::var("APPDATA").unwrap())
				.join("Blender Foundation/Blender/4.2/extensions/user_default")
		} else if cfg!(unix) {
			PathBuf::from(env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
				let mut config = env::var("HOME").unwrap();
				config.push_str("/.config");
				config
			}))
			.join("blender/4.2/extensions/user_default")
		} else {
			return eprintln!("{}", unsupported_platform);
		};
		if !data_dir.exists() {
			fs::create_dir_all(&data_dir).unwrap();
		}

		// Manage the blender extension folder
		data_dir.push("emtk");
		if !data_dir.exists() {
			fs::create_dir(&data_dir).unwrap();
		} else {
			fs::remove_dir_all(&data_dir).unwrap();
			fs::create_dir(&data_dir).unwrap();
		}

		// Copy emtk's blender extension folder to blender's extension folder
		fn copy(source: &Path, target: &Path) {
			for entry in source
				.read_dir()
				.expect("error while reading emtk blender extension folder")
				.flatten()
			{
				let path = entry.path();
				if path.is_dir() {
					fs::create_dir(target.join(path.file_name().unwrap())).unwrap();
					copy(&path, &target.join(path.file_name().unwrap()));
					continue;
				} else if path.is_file() {
					fs::copy(&path, target.join(path.file_name().unwrap())).unwrap();
				}
			}
		}
		copy(
			&project_root.join("bindings/python/emtk-py/emtk/blender"),
			&data_dir,
		);
		println!("Re-toggle the extension inside blender");
	}
}

fn python(build_mode: BuildMode) {
	setup_python();

	let maturin_cmd = "maturin";
	let maturin_args = match build_mode {
		BuildMode::Release => vec![
			"develop",
			"-r",
			"--uv",
			"-m",
			"./bindings/python/emtk-py/Cargo.toml",
		],
		BuildMode::Dev => vec![
			"develop",
			"--uv",
			"-m",
			"./bindings/python/emtk-py/Cargo.toml",
		],
	};

	process::Command::new(maturin_cmd)
		.args(&maturin_args)
		.status()
		.unwrap_or_else(|e| panic_command(maturin_cmd, Some(&maturin_args), e));
}

/// Run all example plugins
fn run() {
	let cargo = cargo_env();
	let project_root = project_root();
	let examples_path = project_root.join("examples");
	let exe_path = exe_path();

	let cargo_build_args = &["build", "-p", "emf"];
	let cargo_run_args = &["run", "-p", "emtk"];

	process::Command::new(&cargo)
		.current_dir(project_root)
		.args(cargo_build_args)
		.status()
		.unwrap_or_else(|e| panic_command(&cargo, Some(cargo_build_args), e));

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

	process::Command::new(&cargo)
		.current_dir(project_root)
		.args(cargo_run_args)
		.status()
		.unwrap_or_else(|e| panic_command(&cargo, Some(cargo_run_args), e));
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

	// Skip when using "cargo xtask run"
	if exanima_exe_path.is_none() {
		let cargo_build_args = &["build", "-p", "emf"];
		process::Command::new(&cargo)
			.current_dir(project_root)
			.args(cargo_build_args)
			.status()
			.unwrap_or_else(|e| panic_command(&cargo, Some(cargo_build_args), e));
	}

	let cargo_build_args = &["build", "-p", name];
	process::Command::new(&cargo)
		.current_dir(project_root)
		.args(cargo_build_args)
		.status()
		.unwrap_or_else(|e| panic_command(&cargo, Some(cargo_build_args), e));

	fs::create_dir_all(plugin_path.clone())
		.unwrap_or_else(|e| panic!("error while creating {} folder at mods path: {}", name, e));
	fs::copy(
		build_path.join(format!("{}.dll", name.replace("-", "_"))),
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
		let cargo_run_args = &["run", "-p", "emtk"];
		process::Command::new(&cargo)
			.current_dir(project_root)
			.args(cargo_run_args)
			.status()
			.unwrap_or_else(|e| panic_command(&cargo, Some(cargo_run_args), e));
	}
}

/// Builds the python wheel for emtk-py.
///
/// # Panics
///
/// - The uv command **must** be in the PATH environment variable
fn wheel(build_mode: BuildMode) {
	setup_python();

	let project_root = project_root();
	let maturin_cmd = "maturin";
	let build_args = match build_mode {
		BuildMode::Release => vec!["build", "-r", "-m", "./bindings/python/emtk-py/Cargo.toml"],
		BuildMode::Dev => vec!["build", "-m", "./bindings/python/emtk-py/Cargo.toml"],
	};

	process::Command::new(maturin_cmd)
		.current_dir(project_root)
		.args(&build_args)
		.status()
		.unwrap_or_else(|e| panic_command(maturin_cmd, Some(&build_args), e));
}
