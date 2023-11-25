// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;
use std::fs::{copy, create_dir_all, read_dir};
use std::io::{self, ErrorKind};
use std::path::PathBuf;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Export DLL Ordinals
    let mut ordinal = project_root().unwrap();
    ordinal.push("emf");
    ordinal.push("lib.def");
    println!("cargo:rustc-cdylib-link-arg=/DEF:{}", ordinal.display());

    // Add deps to search path
    let mut libmem_dll = project_root().unwrap();
    libmem_dll.push("emf");
    libmem_dll.push("deps");
    println!("cargo:rustc-link-search=native={}", libmem_dll.display());

    // Link libmem
    println!("cargo:rustc-link-lib=static=libmem");

    libmem_dll.push("libmem.dll");

    // Copy libmem.dll to target dir.
    let mut destination = project_root().unwrap();
    destination.push("target");
    destination.push(env::var("TARGET").unwrap());
    destination.push(env::var("PROFILE").unwrap());
    if !destination.exists() {
        create_dir_all(&destination)?;
    }
    destination.push("libmem.dll");
    copy(&libmem_dll, destination).unwrap();

    let mut dist_dir = project_root().unwrap();
    dist_dir.push("target");
    dist_dir.push("dist");

    // Cargo, annoyingly, doesn't expose any way to mark "libmem.dll" as a build artefact.
    // It also doesn't give us a way to check where the dist folder is dynamically.
    // This unfortunately means that if this isn't build with `--out-dir`, it will make the
    // target/dist folder already. And if `--out-dir` is set to something else, it will still
    // be created in target/dist.
    if !dist_dir.exists() {
        create_dir_all(&dist_dir)?;
    }

    // Copy libmem.dll to target/dist.
    dist_dir.push("libmem.dll");
    copy(&libmem_dll, dist_dir).unwrap();

    Ok(())
}

fn project_root() -> io::Result<PathBuf> {
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
