use std::{borrow::BorrowMut, env, ffi::CString, io, mem::MaybeUninit, ptr};

use detours_sys::{DetourCreateProcessWithDllExA, _PROCESS_INFORMATION, _STARTUPINFOA};
use winapi::um::{handleapi::CloseHandle, processthreadsapi::ResumeThread};

/// Inject a DLL into a target process.
///
/// # Safety
///
/// This function is unsafe because it is injecting a DLL into a live process.
pub unsafe fn inject(dll_path: &str, target_exe: &str) -> Result<(), emcore::error::Io> {
	let binding = CString::new(target_exe).map_err(|source| emcore::error::Io {
		message: "failed to create new C string for the target executable",
		source: source.into(),
	})?;
	let mut target_exe = binding.as_c_str();
	let dll_path = CString::new(dll_path).map_err(|source| emcore::error::Io {
		message: "failed to create new C string for the dll path",
		source: source.into(),
	})?;

	let mut process_info: _PROCESS_INFORMATION = MaybeUninit::zeroed().assume_init();
	let mut startup_info: _STARTUPINFOA = MaybeUninit::zeroed().assume_init();

	let mut curr_exe_path = env::current_exe().map_err(|source| emcore::error::Io {
		message: "failed to get the path to the currently running executable file",
		source,
	})?;
	curr_exe_path.pop();

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
		return Err(emcore::error::Io {
			message: "failed to create process with dll from detours",
			source: io::Error::last_os_error(),
		});
	}

	ResumeThread(process_info.hThread as _);
	CloseHandle(process_info.hProcess as _);
	CloseHandle(process_info.hThread as _);

	Ok(())
}
