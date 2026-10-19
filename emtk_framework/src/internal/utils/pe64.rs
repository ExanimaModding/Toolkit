use pelite::pe64::PeView;
use windows::Wdk::Storage::FileSystem::NtCreateSection;
use windows::Wdk::System::Memory::{NtMapViewOfSection, NtUnmapViewOfSection, ViewUnmap};
use windows::Win32::Foundation::{HANDLE, HMODULE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, SEC_COMMIT, SECTION_ALL_ACCESS,
};
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::core::{Error, PCSTR};

use std::ptr::{null, null_mut};

pub struct PE64;

impl PE64 {
    #[allow(unused)]
    pub unsafe fn get_base_address() -> Result<HMODULE, Error> {
        unsafe { GetModuleHandleA(PCSTR(null())) }
    }

    pub unsafe fn get_module_information() -> PeView<'static> {
        unsafe {
            let base_address = Self::get_base_address().expect("Get base address");
            PeView::module(base_address.0.cast())
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
    new_permissions: PAGE_PROTECTION_FLAGS,
) -> Result<(), String> {
    // Read section into copy_buf

    unsafe {
        let mut copy_buf = vec![0u8; section_size];
        let mut bytes_read = 0;
        ReadProcessMemory(
            GetCurrentProcess(),
            base_addr.0,
            copy_buf.as_mut_ptr() as _,
            section_size,
            Some(&mut bytes_read),
        )
        .map_err(|e| format!("ReadProcessMemory failed: {:#08x}", e.code().0))?;

        // Create a new template section with the same size as the old section
        // but with our new permissions.

        let mut h_section: HANDLE = HANDLE(null_mut());
        let mut section_max_size = section_size as i64;

        let status = NtCreateSection(
            &mut h_section,
            SECTION_ALL_ACCESS.0,
            None,
            Some(&raw mut section_max_size),
            PAGE_EXECUTE_READWRITE.0,
            SEC_COMMIT.0,
            None,
        );

        if status.is_err() {
            return Err(format!("NtCreateSection failed: {:#08x}", status.0));
        }

        // Unmap the original section

        let status = NtUnmapViewOfSection(GetCurrentProcess(), Some(base_addr.0));

        if status.is_err() {
            return Err(format!("NtUnmapViewOfSection failed: {:#08x}", status.0));
        }

        // Map the new template section into the original section's address space

        let mut view_base = base_addr;
        let mut section_offset: i64 = std::mem::zeroed();
        let mut view_size: usize = 0;

        let status = NtMapViewOfSection(
            h_section as _,
            GetCurrentProcess() as _,
            &mut view_base as *mut _ as _,
            0,
            view_size as _,
            Some(&mut section_offset as *mut _ as _),
            &mut view_size as *mut _ as _,
            ViewUnmap,
            0,
            new_permissions.0,
        );

        if status.is_err() {
            return Err(format!("NtMapViewOfSection failed: {:#08x}", status.0));
        }

        // Write the original section's data into the new section

        let mut bytes_written = 0;
        WriteProcessMemory(
            GetCurrentProcess(),
            base_addr.0,
            copy_buf.as_ptr() as _,
            section_size,
            Some(&mut bytes_written),
        )
        .map_err(|e| format!("WriteProcessMemory failed: {:#08x}", e.code().0))?;
    }

    Ok(())
}
