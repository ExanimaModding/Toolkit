pub mod pe32;

use winapi::{
    shared::minwindef::{BOOL, DWORD, LPVOID},
    um::memoryapi::VirtualProtect,
};

use self::pe32::PE32;

pub unsafe fn virtual_protect(from: LPVOID, size: usize, permissions: DWORD) -> BOOL {
    VirtualProtect(from, size, permissions, &mut 0)
}

pub unsafe fn virtual_protect_module(permissions: DWORD) -> BOOL {
    let h_module_info = PE32::get_module_information();
    virtual_protect(
        h_module_info.lpBaseOfDll,
        h_module_info.SizeOfImage as usize,
        permissions,
    )
}
