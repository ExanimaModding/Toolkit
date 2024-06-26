pub mod injector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	unsafe {
		injector::inject(
			r"emf.dll",
			r"m:\Games\Steam Library\steamapps\common\Exanima\Exanima.exe",
		)
		.unwrap();
	}
	Ok(())
}
