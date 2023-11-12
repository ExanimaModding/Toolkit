use std::env;
use std::fs::read_dir;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

pub fn main() {
    let mut project_root = get_project_root().unwrap();
    project_root.push("deps");
    println!("cargo:rustc-link-search=native={}", project_root.display());
    println!("cargo:rustc-link-lib=static=libmem");
}

fn get_project_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let path_ancestors = path.as_path().ancestors();

    for p in path_ancestors {
        let has_cargo = read_dir(p)?.any(|p| p.unwrap().file_name() == *"Cargo.lock");
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "Ran out of places to find Cargo.toml",
    ))
}
