// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

pub mod metadata;
pub mod types;
pub mod utils;

use crate::types::rpk::RPK;
use std::path::PathBuf;

pub async fn pack(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn unpack(src: &str, dest: &str) -> Result<(), std::io::Error> {
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
