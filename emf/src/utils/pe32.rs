use std::ptr::null_mut;

use exe::{Buffer, PtrPE, VecPE, PE};
use winapi::{
    shared::minwindef::DWORD,
    um::{
        libloaderapi::GetModuleHandleA,
        processthreadsapi::GetCurrentProcess,
        psapi::{GetModuleInformation, MODULEINFO},
    },
};

pub struct PE32;

impl PE32 {
    #[allow(unused)]
    pub unsafe fn get_base_address() -> DWORD {
        GetModuleHandleA(null_mut()) as DWORD
    }

    pub unsafe fn get_entrypoint() -> DWORD {
        let pe_mem = PtrPE::from_memory(0x400000 as _).unwrap();
        let pe = VecPE::from_memory_data(pe_mem.as_slice());

        let headers = pe.get_nt_headers_32().unwrap();
        let optional_headers = headers.optional_header;
        let entrypoint = optional_headers.address_of_entry_point.0;

        // TODO: Auto calculate ImageBase.
        (0x400000 + entrypoint) as _
    }

    pub unsafe fn get_module_information() -> MODULEINFO {
        let mut h_module: MODULEINFO = std::mem::zeroed();
        GetModuleInformation(
            GetCurrentProcess(),
            GetModuleHandleA(null_mut()),
            &mut h_module,
            std::mem::size_of::<MODULEINFO>() as u32,
        );
        h_module
    }
}
