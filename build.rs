pub fn main() {
	println!("cargo:rustc-link-lib=kernel32");
	println!("cargo:rustc-link-lib=ucrt");
}
