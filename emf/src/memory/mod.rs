// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

pub mod asm;
pub mod sigscanner;

use winapi::shared::minwindef::DWORD;
use winapi::um::memoryapi::WriteProcessMemory;
use winapi::um::processthreadsapi::GetCurrentProcess;

use self::sigscanner::SigScanner;

trait AsPtr<T> {
	fn as_ptr(&self) -> *const T;
	fn as_mut_ptr(&mut self) -> *mut T;
}

trait AsNum<T> {
	fn as_num(&self) -> T;
}

pub struct Ptr;

#[allow(unused)]
impl Ptr {
	pub fn as_const<T>(ptr: DWORD) -> *const T {
		ptr as *const T
	}

	pub fn as_mut<T>(ptr: DWORD) -> *mut T {
		ptr as *mut T
	}

	pub fn as_i32(ptr: *const DWORD) -> i32 {
		ptr as i32
	}

	pub unsafe fn deref(ptr: DWORD) -> *mut DWORD {
		*(ptr as *mut *mut DWORD)
	}

	pub fn offset<T>(ptr: DWORD, offset: i32) -> *mut T {
		(ptr as i32 + offset) as *mut T
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

pub struct MemPatch;

impl MemPatch {
	pub unsafe fn many(sig: &str, size: usize, replace: &mut [u8]) -> Result<(), String> {
		loop {
			let addr = SigScanner::new(sig).exec();
			if let Some(addr) = addr.value() {
				let addr = addr as *mut [u8; 6];
				WriteProcessMemory(
					GetCurrentProcess() as _,
					addr as _,
					replace.as_mut_ptr() as _,
					size,
					&mut 0,
				);
				println!("Wrote to addr {:#08x}", addr as u32);
			} else {
				break;
			}
		}
		Ok(())
	}
}
