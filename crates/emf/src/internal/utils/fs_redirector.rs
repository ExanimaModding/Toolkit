use std::{
	ffi::{c_void, CStr},
	path::PathBuf,
	sync::LazyLock,
};

use detours_sys::{DetourAttach, DetourTransactionBegin, DetourTransactionCommit};
use winapi::{
	shared::{minwindef::DWORD, ntdef::LPCSTR},
	um::{
		fileapi::{CreateFileA, CreateFileW},
		minwinbase::LPSECURITY_ATTRIBUTES,
		winnt::HANDLE,
	},
};

static mut O_CREATE_FILE_W: *mut c_void = 0 as _;
static mut O_CREATE_FILE_A: *mut c_void = 0 as _;

pub unsafe fn register_hooks() {
	O_CREATE_FILE_W = CreateFileW as *mut c_void;
	O_CREATE_FILE_A = CreateFileA as *mut c_void;

	DetourTransactionBegin();
	DetourAttach(&raw mut O_CREATE_FILE_A, create_file_a as _);
	DetourTransactionCommit();
}

type TCreateFileA = unsafe extern "system" fn(
	lp_file_name: LPCSTR,
	dw_desired_access: DWORD,
	dw_share_mode: DWORD,
	lp_security_attributes: LPSECURITY_ATTRIBUTES,
	dw_creation_disposition: DWORD,
	dw_flags_and_attributes: DWORD,
	h_template_file: HANDLE,
) -> HANDLE;

unsafe fn create_file_a(
	lp_file_name: LPCSTR,
	dw_desired_access: DWORD,
	dw_share_mode: DWORD,
	lp_security_attributes: LPSECURITY_ATTRIBUTES,
	dw_creation_disposition: DWORD,
	dw_flags_and_attributes: DWORD,
	h_template_file: HANDLE,
) -> HANDLE {
	static CREATE_FILE_A: LazyLock<TCreateFileA> =
		LazyLock::new(|| unsafe { std::mem::transmute(O_CREATE_FILE_A) });

	// Statically read the cwd and cache dir so that it doesn't run every time CreateFileA is called.
	static CWD_PATH: LazyLock<PathBuf> = LazyLock::new(|| std::env::current_dir().unwrap());

	static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
		CWD_PATH
			.clone()
			.join(emcore::Instance::DATA_DIR)
			.join(emcore::Instance::CACHE_DIR)
			.join(emcore::Instance::CACHE_BUILD_DIR)
	});

	// Convert the string pointer to a Rust path.
	let file_name = CStr::from_ptr(lp_file_name).to_string_lossy().into_owned();
	let file_path = std::path::Path::new(&file_name);

	let is_in_cwd = file_path.starts_with(&*CWD_PATH);

	// TODO: Should we restrict or allowlist the file (types) that can be redirected?
	if !is_in_cwd {
		return CREATE_FILE_A(
			lp_file_name,
			dw_desired_access,
			dw_share_mode,
			lp_security_attributes,
			dw_creation_disposition,
			dw_flags_and_attributes,
			h_template_file,
		);
	}

	// Strip the cwd path from the file path, to get the relative path.
	let file_name_stripped = file_path.strip_prefix(&*CWD_PATH).unwrap();
	let file_name_stripped = file_name_stripped.to_str().unwrap();

	// Prefix the relative path with the cache dir.
	let new_file_name = CACHE_DIR.join(file_name_stripped);

	// If the file doesn't exist in cache, fallback to the original file.
	if !new_file_name.exists() {
		return CREATE_FILE_A(
			lp_file_name,
			dw_desired_access,
			dw_share_mode,
			lp_security_attributes,
			dw_creation_disposition,
			dw_flags_and_attributes,
			h_template_file,
		);
	}

	// Convert the new file name to a CString, ensuring it is null-terminated.
	let new_file_name = match std::ffi::CString::new(new_file_name.to_string_lossy().as_bytes()) {
		Ok(cstring) => cstring,
		Err(_) => {
			// If conversion fails, fallback to the original file.
			return CREATE_FILE_A(
				lp_file_name,
				dw_desired_access,
				dw_share_mode,
				lp_security_attributes,
				dw_creation_disposition,
				dw_flags_and_attributes,
				h_template_file,
			);
		}
	};

	// Call the original CreateFileA with the new file name, and return the result.
	CREATE_FILE_A(
		new_file_name.as_ptr(),
		dw_desired_access,
		dw_share_mode,
		lp_security_attributes,
		dw_creation_disposition,
		dw_flags_and_attributes,
		h_template_file,
	)
}
