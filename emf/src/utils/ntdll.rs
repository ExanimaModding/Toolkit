// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use detours_sys::DWORD;
use winapi::um::winnt::HANDLE;

#[link(name = "ntdll")]
#[allow(unused)]
extern "system" {
    pub fn NtProtectVirtualMemory(
        ProcessHandle: DWORD,
        BaseAddress: *mut DWORD,
        NumberOfBytesToProtect: *mut DWORD,
        NewAccessProtection: DWORD,
        OldAccessProtection: *mut DWORD,
    ) -> DWORD;

    pub fn NtOpenProcess(
        ProcessHandle: *mut DWORD,
        DesiredAccess: DWORD,
        ObjectAttributes: *mut DWORD,
        ClientId: *mut DWORD,
    ) -> DWORD;

    pub fn NtQueryVirtualMemory(
        ProcessHandle: DWORD,
        BaseAddress: DWORD,
        MemoryInformationClass: DWORD,
        MemoryInformation: *mut DWORD,
        MemoryInformationLength: DWORD,
        ReturnLength: *mut DWORD,
    ) -> DWORD;

    pub fn NtCreateSection(
        SectionHandle: *mut DWORD,
        DesiredAccess: DWORD,
        ObjectAttributes: *mut DWORD,
        MaximumSize: *mut DWORD,
        SectionPageProtection: DWORD,
        AllocationAttributes: DWORD,
        FileHandle: DWORD,
    ) -> DWORD;

    pub fn NtUnmapViewOfSection(ProcessHandle: HANDLE, BaseAddress: HANDLE) -> DWORD;

    pub fn NtMapViewOfSection(
        SectionHandle: *mut DWORD,
        ProcessHandle: *mut DWORD,
        BaseAddress: *mut DWORD,
        ZeroBits: DWORD,
        CommitSize: DWORD,
        SectionOffset: *mut DWORD,
        ViewSize: *mut DWORD,
        InheritDisposition: DWORD,
        AllocationType: DWORD,
        Win32Protect: DWORD,
    ) -> DWORD;

    pub fn ZwProtectVirtualMemory(
        ProcessHandle: HANDLE,
        BaseAddress: HANDLE,
        NumberOfBytesToProtect: *mut DWORD,
        NewAccessProtection: DWORD,
        OldAccessProtection: *mut DWORD,
    ) -> DWORD;

}

#[repr(u32)]
pub enum NtStatus {
    Success = 0x00000000,
    Other(u32),
}

impl From<DWORD> for NtStatus {
    fn from(value: u32) -> Self {
        match value {
            0x00000000 => NtStatus::Success,
            _ => NtStatus::Other(value),
        }
    }
}
