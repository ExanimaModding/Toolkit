// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use detours_sys::{_PROCESS_INFORMATION, _STARTUPINFOA};
use std::borrow::BorrowMut;
use std::ptr::{null, null_mut};
use std::{ffi::CString, mem::MaybeUninit};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::ResumeThread;

/// Inject a DLL into a target process.
///
/// # Safety
///
/// This function is unsafe because it is injecting a DLL into a live process.
pub unsafe fn inject(dll_path: &str, target_exe: &str) -> std::io::Result<()> {
	let binding = CString::new(target_exe)?;
	let mut target_exe = binding.as_c_str();
	let dll_path = CString::new(dll_path).unwrap();

	dbg!(&target_exe, &dll_path);

	let mut process_info: _PROCESS_INFORMATION = MaybeUninit::zeroed().assume_init();
	let mut startup_info: _STARTUPINFOA = MaybeUninit::zeroed().assume_init();

	let mut curr_exe_path = std::env::current_exe().unwrap();
	curr_exe_path.pop();

	let result = detours_sys::DetourCreateProcessWithDllExA(
		null(),
		target_exe.borrow_mut().as_ptr() as _,
		null_mut(),
		null_mut(),
		0,
		0,
		null_mut(),
		null(),
		&mut startup_info as *mut _,
		&mut process_info as *mut _,
		dll_path.as_ptr() as _,
		None,
	);

	if result == 0 {
		eprintln!("CreateProcessA failed: {}", result);
		return Err(std::io::Error::last_os_error());
	}

	ResumeThread(process_info.hThread as _);
	CloseHandle(process_info.hProcess as _);
	CloseHandle(process_info.hThread as _);

	Ok(())
}
