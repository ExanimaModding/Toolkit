use std::ptr::null_mut;

use winapi::{
	shared::{
		basetsd::SIZE_T,
		minwindef::{BOOL, DWORD},
		ntdef::{HANDLE, NTSTATUS},
	},
	um::{
		errhandlingapi::GetLastError,
		libloaderapi::GetModuleHandleA,
		memoryapi::{ReadProcessMemory, WriteProcessMemory},
		processthreadsapi::GetCurrentProcess,
		winnt::{LARGE_INTEGER, PAGE_EXECUTE_READWRITE, SEC_COMMIT, SECTION_ALL_ACCESS},
	},
};

use pelite::pe64::PeView;

use crate::internal::utils::ntdll::NtStatus;

use super::ntdll::{NtCreateSection, NtMapViewOfSection, NtUnmapViewOfSection};

pub struct PE64;

impl PE64 {
	#[allow(unused)]
	pub unsafe fn get_base_address() -> usize {
		unsafe { GetModuleHandleA(null_mut()) as _ }
	}

	pub unsafe fn get_module_information() -> PeView<'static> {
		unsafe {
			let base_address = Self::get_base_address();
			PeView::module(base_address as _)
		}
	}
}

/// Remap a section of memory with new permissions.
///
/// # Caution
/// This function will delete the old section and recreate it with new permissions.
pub unsafe fn remap_view_of_section(
	base_addr: HANDLE,
	section_size: usize,
	new_permissions: DWORD,
) -> Result<(), String> {
	// Read section into copy_buf

	unsafe {
		let mut copy_buf = vec![0u8; section_size];
		let mut bytes_read = 0;
		let success: BOOL = ReadProcessMemory(
			GetCurrentProcess(),
			base_addr,
			copy_buf.as_mut_ptr() as _,
			section_size,
			&mut bytes_read,
		);

		if success == 0 {
			return Err(format!("ReadProcessMemory failed: {:#08x}", GetLastError()));
		}

		// Create a new template section with the same size as the old section
		// but with our new permissions.

		let mut h_section: HANDLE = 0 as _;
		let mut section_max_size: LARGE_INTEGER = std::mem::zeroed();
		*section_max_size.QuadPart_mut() = section_size as _;

		let success: NTSTATUS = NtCreateSection(
			&mut h_section,
			SECTION_ALL_ACCESS,
			null_mut(),
			&raw mut section_max_size as _,
			PAGE_EXECUTE_READWRITE,
			SEC_COMMIT,
			std::ptr::null_mut(),
		);

		if let NtStatus::Other(val) = NtStatus::from(success) {
			return Err(format!("NtCreateSection failed: {:#08x}", val));
		}

		// Unmap the original section

		let success: NTSTATUS = NtUnmapViewOfSection(GetCurrentProcess(), base_addr);

		if let NtStatus::Other(val) = NtStatus::from(success) {
			return Err(format!("NtUnmapViewOfSection failed: {:#08x}", val));
		}

		// Map the new template section into the original section's address space

		let mut view_base = base_addr;
		let mut section_offset: LARGE_INTEGER = std::mem::zeroed();
		let mut view_size: SIZE_T = 0;

		let success: NTSTATUS = NtMapViewOfSection(
			h_section as _,
			GetCurrentProcess() as _,
			&mut view_base as *mut _ as _,
			0,
			view_size as _,
			&mut section_offset as *mut _ as _,
			&mut view_size as *mut _ as _,
			2, // ViewUnmap
			0,
			new_permissions,
		);

		if let NtStatus::Other(val) = NtStatus::from(success) {
			return Err(format!("NtMapViewOfSection failed: {:#08x}", val));
		}

		// Write the original section's data into the new section

		let mut bytes_written = 0;
		let success: BOOL = WriteProcessMemory(
			GetCurrentProcess(),
			base_addr,
			copy_buf.as_ptr() as _,
			section_size,
			&mut bytes_written,
		);

		if success == 0 {
			return Err(format!(
				"WriteProcessMemory failed: {:#08x}",
				GetLastError()
			));
		}
	}

	Ok(())
}
