// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::null_mut;

use exe::{Buffer, PtrPE, VecPE, PE};
use winapi::{
    ctypes::c_void,
    shared::{
        basetsd::SIZE_T,
        minwindef::{BOOL, DWORD},
    },
    um::{
        errhandlingapi::GetLastError,
        libloaderapi::GetModuleHandleA,
        memoryapi::{ReadProcessMemory, WriteProcessMemory},
        processthreadsapi::GetCurrentProcess,
        winnt::{LARGE_INTEGER, SECTION_ALL_ACCESS, SEC_COMMIT},
    },
};

use pelite::pe32::PeView;

use crate::utils::ntdll::NtStatus;

use super::ntdll::{NtCreateSection, NtMapViewOfSection, NtUnmapViewOfSection};

pub struct PE32;

impl PE32 {
    #[allow(unused)]
    pub unsafe fn get_base_address() -> DWORD {
        GetModuleHandleA(null_mut()) as DWORD
    }

    #[allow(unused)]
    pub unsafe fn get_entrypoint() -> DWORD {
        let pe_mem = PtrPE::from_memory(0x400000 as _).unwrap();
        let pe = VecPE::from_memory_data(pe_mem.as_slice());

        let headers = pe.get_nt_headers_32().unwrap();
        let optional_headers = headers.optional_header;
        let entrypoint = optional_headers.address_of_entry_point.0;

        // TODO: Auto calculate ImageBase.
        (0x400000 + entrypoint) as _
    }

    pub unsafe fn get_module_information() -> PeView<'static> {
        let base_address = Self::get_base_address();
        PeView::module(base_address as _)
    }
}

/// Remap a section of memory with new permissions.
///
/// # Caution
/// This function will delete the old section and recreate it with new permissions.
pub unsafe fn remap_view_of_section(
    base_addr: *mut c_void,
    section_size: usize,
    new_permissions: DWORD,
) -> Result<(), String> {
    // Read section into copy_buf

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

    let mut h_section: DWORD = 0 as _;
    let mut section_max_size: LARGE_INTEGER = std::mem::zeroed();
    *section_max_size.QuadPart_mut() = section_size as _;

    let success: DWORD = NtCreateSection(
        &mut h_section as _,
        SECTION_ALL_ACCESS,
        null_mut(),
        &mut section_max_size as *mut _ as _,
        new_permissions,
        SEC_COMMIT,
        0,
    );

    if let NtStatus::Other(val) = NtStatus::from(success) {
        return Err(format!("NtCreateSection failed: {:#08x}", val));
    }

    // Unmap the original section

    let success: DWORD = NtUnmapViewOfSection(GetCurrentProcess(), base_addr);

    if let NtStatus::Other(val) = NtStatus::from(success) {
        return Err(format!("NtUnmapViewOfSection failed: {:#08x}", val));
    }

    // Map the new template section into the original section's address space

    let mut view_base = base_addr;
    let mut section_offset: LARGE_INTEGER = std::mem::zeroed();
    let mut view_size: SIZE_T = 0;

    let success: DWORD = NtMapViewOfSection(
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

    Ok(())
}
