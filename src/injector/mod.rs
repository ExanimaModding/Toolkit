// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use std::{ffi::CString, mem::MaybeUninit};

use std::ptr::null_mut;

use dll_syringe::Syringe;

use winapi::um::handleapi::CloseHandle;

use winapi::um::processthreadsapi::{
    CreateProcessA, ResumeThread, PROCESS_INFORMATION, STARTUPINFOA,
};
use winapi::um::winbase::CREATE_SUSPENDED;

use dll_syringe::process::OwnedProcess;

/// # Safety
/// This function is unsafe because it calls the Windows API.
pub unsafe fn inject(dll_path: &str, target_exe: &str) -> std::io::Result<()> {
    let cstr_target_exe = CString::new(target_exe)?;

    let mut process_info: PROCESS_INFORMATION = MaybeUninit::zeroed().assume_init();
    let mut startup_info: STARTUPINFOA = MaybeUninit::zeroed().assume_init();

    // This is required for Steam to recognize the game as running.
    // Only do this in release builds so that it doesn't spam "now playing".
    #[cfg(not(debug_assertions))]
    std::env::set_var("SteamAppId", "362490");

    let result = CreateProcessA(
        null_mut(),
        cstr_target_exe.as_ptr() as _,
        null_mut(),
        null_mut(),
        0,
        CREATE_SUSPENDED,
        null_mut(),
        null_mut(),
        &mut startup_info as *mut _,
        &mut process_info as *mut _,
    );

    if result != 1 {
        eprintln!("CreateProcessA failed: {}", result);
        return Err(std::io::Error::last_os_error());
    }

    let pproc = OwnedProcess::from_pid(process_info.dwProcessId).unwrap();
    let syringe = Syringe::for_process(pproc);
    syringe.inject(dll_path).unwrap();

    ResumeThread(process_info.hThread);
    CloseHandle(process_info.hProcess);
    CloseHandle(process_info.hThread);

    Ok(())
}
