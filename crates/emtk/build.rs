use winresource::WindowsResource;

fn main() -> std::io::Result<()> {
	if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
		WindowsResource::new()
			// This path can be absolute, or relative to your crate root.
			.set_icon("../../assets/images/corro.ico")
			.compile()?;
	}

	Ok(())
}
