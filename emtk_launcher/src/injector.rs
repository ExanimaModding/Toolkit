use std::{borrow::BorrowMut, env, ffi::CString, io, mem::MaybeUninit, ptr};

use detours_sys::{
	_PROCESS_INFORMATION, _STARTUPINFOA, BOOL, DWORD, DetourCreateProcessWithDllExA, HANDLE,
};
use emtk_core::{Error, Result};
use tracing::instrument;

/// Inject a DLL into a target process.
///
/// # Safety
///
/// This function is unsafe because it is injecting a DLL into a live process.
#[instrument(level = "trace")]
pub(crate) unsafe fn inject(dll_path: &str, target_exe: &str) -> Result<()> {
	let binding = CString::new(target_exe).map_err(Error::msg(
		"failed to create new C string for the target executable",
	))?;
	let mut target_exe = binding.as_c_str();
	let dll_path = CString::new(dll_path)
		.map_err(Error::msg("failed to create new C string for the dll path"))?;

	let mut process_info: _PROCESS_INFORMATION = unsafe { MaybeUninit::zeroed().assume_init() };
	let mut startup_info: _STARTUPINFOA = unsafe { MaybeUninit::zeroed().assume_init() };

	let mut curr_exe_path = env::current_exe().map_err(Error::msg(
		"failed to get the path to the currently running executable file",
	))?;
	curr_exe_path.pop();

	unsafe {
		let result = DetourCreateProcessWithDllExA(
			ptr::null(),
			target_exe.borrow_mut().as_ptr() as _,
			ptr::null_mut(),
			ptr::null_mut(),
			0,
			0,
			ptr::null_mut(),
			ptr::null(),
			&mut startup_info as *mut _,
			&mut process_info as *mut _,
			dll_path.as_ptr() as _,
			None,
		);

		if result == 0 {
			eprintln!("CreateProcessA failed: {}", result);
			return Err(Error::new(
				io::Error::last_os_error(),
				"failed to create process with dll from detours",
			));
		}

		ResumeThread(process_info.hThread as _);
		CloseHandle(process_info.hProcess as _);
		CloseHandle(process_info.hThread as _);
	}

	Ok(())
}

#[link(name = "kernel32")]
unsafe extern "system" {
	unsafe fn CloseHandle(hObject: HANDLE) -> BOOL;

	unsafe fn ResumeThread(hThread: HANDLE) -> DWORD;
}
