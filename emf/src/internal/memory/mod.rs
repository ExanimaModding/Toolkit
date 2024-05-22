// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

pub mod sigscanner;

use log::*;

use winapi::shared::ntdef::DWORDLONG;
use winapi::um::memoryapi::WriteProcessMemory;
use winapi::um::processthreadsapi::GetCurrentProcess;

use self::sigscanner::SigScanner;

#[allow(unused)]
trait AsPtr<T> {
	fn as_ptr(&self) -> *const T;
	fn as_mut_ptr(&mut self) -> *mut T;
}

#[allow(unused)]
trait AsNum<T> {
	fn as_num(&self) -> T;
}

pub struct Ptr;

#[allow(unused)]
impl Ptr {
	pub fn as_const<T>(ptr: DWORDLONG) -> *const T {
		ptr as *const T
	}

	pub fn as_mut<T>(ptr: DWORDLONG) -> *mut T {
		ptr as *mut T
	}

	pub fn as_i32(ptr: *const DWORDLONG) -> i64 {
		ptr as i64
	}

	pub unsafe fn deref(ptr: DWORDLONG) -> *mut DWORDLONG {
		*(ptr as *mut *mut DWORDLONG)
	}

	pub fn offset<T>(ptr: DWORDLONG, offset: i64) -> *mut T {
		(ptr as i64 + offset) as *mut T
	}
}

#[allow(non_camel_case_types, clippy::enum_variant_names)]
pub enum _MEMORY_INFORMATION_CLASS {
	MemoryBasicInformation,          // MEMORY_BASIC_INFORMATION
	MemoryWorkingSetInformation,     // MEMORY_WORKING_SET_INFORMATION
	MemoryMappedFilenameInformation, // UNICODE_STRING
	MemoryRegionInformation,         // MEMORY_REGION_INFORMATION
	MemoryWorkingSetExInformation,   // MEMORY_WORKING_SET_EX_INFORMATION
	MemorySharedCommitInformation,   // MEMORY_SHARED_COMMIT_INFORMATION
	MemoryImageInformation,          // MEMORY_IMAGE_INFORMATION
}
