// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

mod api;
mod memory;
mod utils;

use std::ffi::c_void;

use detours_sys::{
    DetourAttach, DetourIsHelperProcess, DetourRestoreAfterWith, DetourTransactionBegin,
    DetourTransactionCommit, DetourUpdateThread,
};
use pelite::pe32::Pe;
use winapi::{
    shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
    um::{
        consoleapi::AllocConsole,
        processthreadsapi::GetCurrentThread,
        winbase::SetProcessDEPPolicy,
        winnt::{DLL_PROCESS_ATTACH, HANDLE, PAGE_EXECUTE_READWRITE},
    },
};

use crate::utils::pe32::{remap_view_of_section, PE32};

static mut ORIGINAL_START: *mut c_void = 0 as _;

#[no_mangle]
unsafe extern "stdcall" fn DllMain(
    _hinst_dll: HINSTANCE,
    fwd_reason: DWORD,
    _lpv_reserved: LPVOID,
) -> BOOL {
    if DetourIsHelperProcess() != 0 {
        return 1;
    }

    if fwd_reason == DLL_PROCESS_ATTACH {
        AllocConsole();
        println!("[EMF DllMain] DllMain Loaded");

        dbg!(SetProcessDEPPolicy(0));

        let info = PE32::get_module_information().optional_header();
        let page_start = info.ImageBase as HANDLE;
        let page_size = info.SizeOfImage as usize;

        remap_view_of_section(page_start, page_size, PAGE_EXECUTE_READWRITE).unwrap();

        DetourRestoreAfterWith();
        DetourTransactionBegin();
        DetourUpdateThread(GetCurrentThread() as _);

        let opt_headers = PE32::get_module_information().optional_header();
        ORIGINAL_START = (opt_headers.ImageBase + opt_headers.AddressOfEntryPoint) as _;
        DetourAttach(&mut ORIGINAL_START as _, main as _);

        DetourTransactionCommit();

        println!("[EMF] Hooked Entrypoint");
    }

    1
}

#[no_mangle]
unsafe extern "C" fn main() {
    println!("[EMF] Main Loaded");
    api::init_api();

    let original_start: extern "C" fn() = std::mem::transmute(ORIGINAL_START);
    original_start();
}
