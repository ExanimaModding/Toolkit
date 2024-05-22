// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use winapi::um::{
	memoryapi::VirtualQueryEx,
	winnt::{HANDLE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, PAGE_READWRITE},
};

use log::*;

pub mod exports;
pub mod hook;
pub mod patch;

pub unsafe fn init_api() {
	info!("[EMF] API Initialized");
}

pub unsafe fn location_is_readwrite(address: HANDLE, proc: HANDLE) -> anyhow::Result<()> {
	let mut mem_info: MEMORY_BASIC_INFORMATION = std::mem::MaybeUninit::zeroed().assume_init();

	// TODO: Check if code section, etc.

	let result = VirtualQueryEx(
		proc as _,
		address as _,
		&raw mut mem_info,
		std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
	);

	if result == 0 {
		return Err(anyhow::anyhow!(
			"VirtualQueryEx failed on memory location {:p}.",
			address
		));
	}

	if mem_info.AllocationProtect & PAGE_EXECUTE_READWRITE != PAGE_EXECUTE_READWRITE
		&& mem_info.AllocationProtect & PAGE_READWRITE != PAGE_READWRITE
	{
		return Err(anyhow::anyhow!(
			"Memory location {:p} is not writeable.",
			address
		));
	}

	Ok(())
}
