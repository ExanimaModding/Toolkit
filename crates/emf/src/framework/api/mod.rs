pub(crate) mod exports;
pub(crate) mod hook;
pub(crate) mod patch;
pub(crate) mod plugin;

use std::mem;

use winapi::um::{
	memoryapi::VirtualQueryEx,
	winnt::{HANDLE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, PAGE_READWRITE},
};

pub unsafe fn location_is_readwrite(address: HANDLE, proc: HANDLE) -> anyhow::Result<()> {
	let mut mem_info: MEMORY_BASIC_INFORMATION =
		unsafe { mem::MaybeUninit::zeroed().assume_init() };

	// TODO: Check if code section, etc.

	let result = unsafe {
		VirtualQueryEx(
			proc as _,
			address as _,
			&raw mut mem_info,
			std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
		)
	};

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
