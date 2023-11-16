// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;
use std::fs::read_dir;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

pub fn main() {
    // TODO: Statically link libmem
    let mut project_root = get_project_root().unwrap();
    project_root.push("deps");
    println!("cargo:rustc-link-search=native={}", project_root.display());
    println!("cargo:rustc-link-lib=static=libmem");

    let mut destination = PathBuf::from("target");
    destination.push(env::var("TARGET").unwrap());
    destination.push(env::var("PROFILE").unwrap());

    let mut libmem_dll = project_root.clone();
    libmem_dll.push("libmem.dll");

    std::fs::copy(libmem_dll, destination.join("libmem.dll")).unwrap();
    println!("{:?}", destination);
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
