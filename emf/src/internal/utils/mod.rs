// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

pub mod exceptions;
pub mod ntdll;
pub mod pe32;

use std::{ffi::CString, path::PathBuf, ptr::null_mut};

use pelite::pe32::Pe;
use winapi::{
	shared::{
		minwindef::{BOOL, DWORD, LPVOID, MAX_PATH},
		ntdef::HANDLE,
	},
	um::{
		memoryapi::VirtualProtect, processthreadsapi::GetCurrentProcess,
		psapi::GetModuleFileNameExA, winnt::PAGE_EXECUTE_READWRITE,
	},
};

use self::pe32::{remap_view_of_section, PE32};

#[allow(unused)]
pub unsafe fn virtual_protect(
	from: LPVOID,
	size: usize,
	permissions: DWORD,
	old_protect: &mut DWORD,
) -> BOOL {
	VirtualProtect(from, size, permissions, old_protect)
}

#[allow(unused)]
pub unsafe fn virtual_protect_module(permissions: DWORD) -> BOOL {
	let h_module_info = PE32::get_module_information();
	virtual_protect(
		h_module_info.optional_header().ImageBase as _,
		h_module_info.optional_header().SizeOfImage as _,
		permissions,
		&mut 0,
	)
}

pub unsafe fn remap_image() -> Result<(), String> {
	let info = PE32::get_module_information().optional_header();
	let page_start = info.ImageBase as HANDLE;
	let page_size = info.SizeOfImage as usize;

	remap_view_of_section(page_start, page_size, PAGE_EXECUTE_READWRITE)
}

pub fn get_game_path() -> PathBuf {
	let mut path = vec![0_u8; MAX_PATH];
	let path = unsafe {
		let parent_process = GetCurrentProcess();
		GetModuleFileNameExA(
			parent_process,
			null_mut(),
			path.as_mut_ptr() as _,
			MAX_PATH as u32,
		);

		CString::from_raw(path.as_mut_ptr() as _)
			.to_string_lossy()
			.into_owned()
	};

	PathBuf::from(path)
}

pub fn get_game_dir() -> PathBuf {
	let mut path = get_game_path();
	path.pop();
	path
}
