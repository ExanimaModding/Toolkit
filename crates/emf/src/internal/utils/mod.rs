// pub(crate) mod fs_redirector;
pub(crate) mod ntdll;
pub(crate) mod pe64;
pub(crate) mod rpk_intercept;

use pelite::pe::Pe;
use winapi::{
	shared::{
		minwindef::{BOOL, DWORD, LPVOID},
		ntdef::HANDLE,
	},
	um::{memoryapi::VirtualProtect, winnt::PAGE_EXECUTE_READWRITE},
};

use self::pe64::{PE64, remap_view_of_section};

#[allow(unused)]
pub unsafe fn virtual_protect(
	from: LPVOID,
	size: usize,
	permissions: DWORD,
	old_protect: &mut DWORD,
) -> BOOL {
	unsafe { VirtualProtect(from, size, permissions, old_protect) }
}

#[allow(unused)]
pub unsafe fn virtual_protect_module(permissions: DWORD) -> BOOL {
	unsafe {
		let h_module_info = PE64::get_module_information();
		virtual_protect(
			h_module_info.optional_header().ImageBase as _,
			h_module_info.optional_header().SizeOfImage as _,
			permissions,
			&mut 0,
		)
	}
}

pub unsafe fn remap_image() -> Result<(), String> {
	let info = unsafe { PE64::get_module_information().optional_header() };
	let page_start = info.ImageBase as HANDLE;
	let page_size = info.SizeOfImage as usize;

	unsafe { remap_view_of_section(page_start, page_size, PAGE_EXECUTE_READWRITE) }
}
