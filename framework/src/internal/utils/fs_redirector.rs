use detours_sys::{DetourAttach, DetourTransactionBegin, DetourTransactionCommit};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::Storage::FileSystem::{CreateFileA, CreateFileW};
use windows::core::{PCSTR, PCWSTR};

use std::ffi::{CStr, CString, c_void};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{env, mem};

static mut O_CREATE_FILE_W: *mut c_void = 0 as _;
static mut O_CREATE_FILE_A: *mut c_void = 0 as _;

pub unsafe fn register_hooks() {
    unsafe {
        O_CREATE_FILE_W = CreateFileW::<PCWSTR> as *mut c_void;
        O_CREATE_FILE_A = CreateFileA::<PCSTR> as *mut c_void;

        DetourTransactionBegin();
        DetourAttach(&raw mut O_CREATE_FILE_A, create_file_a as _);
        DetourTransactionCommit();
    }
}

type TCreateFileA = unsafe extern "system" fn(
    lp_file_name: *mut PCSTR,
    dw_desired_access: u32,
    dw_share_mode: u32,
    lp_security_attributes: *mut SECURITY_ATTRIBUTES,
    dw_creation_disposition: u32,
    dw_flags_and_attributes: u32,
    h_template_file: HANDLE,
) -> HANDLE;

unsafe fn create_file_a(
    lp_file_name: *mut PCSTR,
    dw_desired_access: u32,
    dw_share_mode: u32,
    lp_security_attributes: *mut SECURITY_ATTRIBUTES,
    dw_creation_disposition: u32,
    dw_flags_and_attributes: u32,
    h_template_file: HANDLE,
) -> HANDLE {
    static CREATE_FILE_A: LazyLock<TCreateFileA> =
        LazyLock::new(|| unsafe { mem::transmute(O_CREATE_FILE_A) });

    // Statically read the cwd and cache dir so that it doesn't run every time CreateFileA is called.
    static CWD_PATH: LazyLock<PathBuf> = LazyLock::new(|| env::current_dir().unwrap());

    static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        CWD_PATH
            .clone()
            .join(emtk::Instance::DATA_DIR)
            .join(emtk::Instance::CACHE_DIR)
            .join(emtk::Instance::CACHE_BUILD_DIR)
    });

    // Convert the string pointer to a Rust path.
    let file_name = unsafe {
        CStr::from_ptr(lp_file_name as _)
            .to_string_lossy()
            .into_owned()
    };
    let file_path = Path::new(&file_name);

    let is_in_cwd = file_path.starts_with(&*CWD_PATH);

    // TODO: Should we restrict or allowlist the file (types) that can be redirected?
    if !is_in_cwd {
        return unsafe {
            CREATE_FILE_A(
                lp_file_name,
                dw_desired_access,
                dw_share_mode,
                lp_security_attributes,
                dw_creation_disposition,
                dw_flags_and_attributes,
                h_template_file,
            )
        };
    }

    // Strip the cwd path from the file path, to get the relative path.
    let file_name_stripped = file_path.strip_prefix(&*CWD_PATH).unwrap();
    let file_name_stripped = file_name_stripped.to_str().unwrap();

    // Prefix the relative path with the cache dir.
    let new_file_name = CACHE_DIR.join(file_name_stripped);

    // If the file doesn't exist in cache, fallback to the original file.
    if !new_file_name.exists() {
        return unsafe {
            CREATE_FILE_A(
                lp_file_name,
                dw_desired_access,
                dw_share_mode,
                lp_security_attributes,
                dw_creation_disposition,
                dw_flags_and_attributes,
                h_template_file,
            )
        };
    }

    // Convert the new file name to a CString, ensuring it is null-terminated.
    let new_file_name = match CString::new(new_file_name.to_string_lossy().as_bytes()) {
        Ok(cstring) => cstring,
        Err(_) => {
            // If conversion fails, fallback to the original file.
            return unsafe {
                CREATE_FILE_A(
                    lp_file_name,
                    dw_desired_access,
                    dw_share_mode,
                    lp_security_attributes,
                    dw_creation_disposition,
                    dw_flags_and_attributes,
                    h_template_file,
                )
            };
        }
    };

    // Call the original CreateFileA with the new file name, and return the result.
    unsafe {
        CREATE_FILE_A(
            PCSTR(new_file_name.as_ptr().cast()).as_ptr() as _,
            dw_desired_access,
            dw_share_mode,
            lp_security_attributes,
            dw_creation_disposition,
            dw_flags_and_attributes,
            h_template_file,
        )
    }
}
