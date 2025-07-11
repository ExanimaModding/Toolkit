use std::{env, io};

use winresource::WindowsResource;

fn main() -> io::Result<()> {
	let workspace_root = env::current_dir().unwrap();
	let workspace_root = workspace_root.ancestors().nth(1).unwrap();
	println!(
		"cargo:rustc-env=WORKSPACE_ROOT={}",
		workspace_root.display()
	);

	if env::var_os("CARGO_CFG_WINDOWS").is_some() {
		WindowsResource::new()
			// This path can be absolute, or relative to your crate root.
			.set_icon(
				&workspace_root
					.join("assets/images/corro.ico")
					.display()
					.to_string(),
			)
			.compile()?;
	}

	Ok(())
}
