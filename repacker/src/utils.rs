// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{metadata::MagicBytes, types::rpk::RPK};
use std::{fs::DirEntry, path::PathBuf};

pub fn red(s: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", s)
}

pub fn green(s: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", s)
}

pub fn yellow(s: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", s)
}

/// # Safety
/// This function is unsafe because it converts any type to a slice of u8.
pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

pub fn is_file_valid(entry: &DirEntry) -> bool {
    if !entry.path().is_file() {
        return false;
    }

    let path = entry.path();

    let ext = path.extension().unwrap();
    let ext_str = ext.to_str().unwrap();
    if MagicBytes::try_from(ext_str).is_err() {
        // Hard coding rcd and rdb until a better
        // solution presents itself
        if !(ext_str == "rcd" || ext_str == "rdb" || ext_str == "unknown") {
            return false;
        }
    }

    true
}

pub async fn pack_all(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = PathBuf::from(src);

    let mut handles = vec![];
    let dest = String::from(dest);
    for entry in src_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dest = dest.clone();
        handles.push(tokio::spawn(async move {
            if let Err(e) = RPK::pack(path.to_str().unwrap(), dest.as_str()) {
                eprintln!("Skipping folder at '{}': {}", path.to_str().unwrap(), e);
            };
        }));
    }
    futures::future::join_all(handles).await;

    Ok(())
}

pub async fn unpack_all(src: &str, dest: &str) -> Result<(), std::io::Error> {
    let src_path = PathBuf::from(src);

    let mut handles = vec![];
    for entry in src_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_str().unwrap();

        if !entry.file_type()?.is_file()
            || !(path_str.ends_with(".fds")
                || path_str.ends_with(".flb")
                || path_str.ends_with(".rml")
                || path_str.ends_with(".rpk"))
        {
            continue;
        }

        let mut dest_path = PathBuf::from(dest);
        dest_path.push(path.with_extension("").file_name().unwrap());

        handles.push(tokio::spawn(async move {
            if let Err(e) = RPK::unpack(path.to_str().unwrap(), dest_path.to_str().unwrap()).await {
                eprintln!("{}", e);
            };
        }));
    }
    futures::future::join_all(handles).await;

    Ok(())
}
