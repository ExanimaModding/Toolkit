pub(crate) mod fs_redirector;
pub(crate) mod pe64;
pub(crate) mod rpk_intercept;

use pelite::pe::Pe;
use windows::Win32::Foundation::{HANDLE, MAX_PATH};
use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect,
};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExA;
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::core::Error;

use std::ffi::{CStr, c_void};
use std::path::PathBuf;

use self::pe64::{PE64, remap_view_of_section};

#[allow(unused)]
pub unsafe fn virtual_protect(
    from: *mut c_void,
    size: usize,
    permissions: PAGE_PROTECTION_FLAGS,
    old_protect: &mut PAGE_PROTECTION_FLAGS,
) -> Result<(), Error> {
    unsafe { VirtualProtect(from, size, permissions, old_protect) }
}

#[allow(unused)]
pub unsafe fn virtual_protect_module(permissions: PAGE_PROTECTION_FLAGS) -> Result<(), Error> {
    unsafe {
        let h_module_info = PE64::get_module_information();
        virtual_protect(
            h_module_info.optional_header().ImageBase as _,
            h_module_info.optional_header().SizeOfImage as _,
            permissions,
            &mut PAGE_PROTECTION_FLAGS(0),
        )
    }
}

pub unsafe fn remap_image() -> Result<(), String> {
    let info = unsafe { PE64::get_module_information().optional_header() };
    let page_start = HANDLE(info.ImageBase as *mut c_void);
    let page_size = info.SizeOfImage as usize;

    unsafe { remap_view_of_section(page_start, page_size, PAGE_EXECUTE_READWRITE) }
}

pub fn get_game_path() -> PathBuf {
    let mut path = vec![0_u8; MAX_PATH as usize];
    let path = unsafe {
        let parent_process = GetCurrentProcess();
        GetModuleFileNameExA(Some(parent_process), None, &mut path);

        CStr::from_ptr(path.as_ptr() as _)
            .to_string_lossy()
            .into_owned()
    };

    PathBuf::from(path)
}

pub fn get_game_dir() -> PathBuf {
    let mut path = get_game_path();
    path.pop();
    path
}
