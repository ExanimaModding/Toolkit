// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use crate::metadata::MagicBytes;
use std::{
	fs::DirEntry,
	io::{Read, Seek},
};

pub trait ReadSeek: Seek + Read + Send + Sync {}

impl<T: Seek + Read + Send + Sync> ReadSeek for T {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SourceData {
	Path(String),
	Buffer(String, Vec<u8>),
}

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
