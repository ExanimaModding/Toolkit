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
    if let Err(_) = MagicBytes::try_from(ext_str) {
        // Hard coding rcd and rdb until a better
        // solution presents itself
        if !(ext_str == "rcd" || ext_str == "rdb" || ext_str == "unknown") {
            return false;
        }
    }

    true
}

pub fn pack_all(src: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = PathBuf::from(src);

    for entry in src_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        if let Err(e) = RPK::pack(path.to_str().unwrap(), dest) {
            eprintln!("Skipping folder at '{}': {}", path.to_str().unwrap(), e);
            continue;
        };
    }

    Ok(())
}

pub async fn unpack_all(src: &str, dest: &str) -> Result<(), std::io::Error> {
    let src_path = PathBuf::from(src);

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

        if let Err(e) = RPK::unpack(path.to_str().unwrap(), dest_path.to_str().unwrap()).await {
            eprintln!("{}", e);
        };
    }

    Ok(())
}
