// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

#![feature(naked_functions, slice_pattern, asm_const)]

mod api;
mod memory;
mod utils;

use std::ffi::c_void;

use detours_sys::{
    DetourAttach, DetourIsHelperProcess, DetourRestoreAfterWith, DetourTransactionBegin,
    DetourTransactionCommit, DetourUpdateThread,
};
use pelite::pe32::Pe;
use winapi::{
    shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
    um::{
        consoleapi::AllocConsole,
        processthreadsapi::GetCurrentThread,
        winbase::SetProcessDEPPolicy,
        winnt::{DLL_PROCESS_ATTACH, HANDLE, PAGE_EXECUTE_READWRITE},
    },
};

use crate::{
    memory::asm::wrap_borland,
    utils::pe32::{remap_view_of_section, PE32},
};

static mut ORIGINAL_START: *mut c_void = 0 as _;

static mut PROCDMGSTAM_WRAPPER: Vec<u8> = Vec::new();
static mut PROCDMGSTAM_ORIG: *mut c_void = 0x0056c64c as _;

static mut CREATE_INSTANCE_WRAPPER: Vec<u8> = Vec::new();
static mut CREATE_INSTANCE_ORIG: *mut c_void = 0x0058b080 as _;

static mut DONE: u8 = 0;
static mut ACTOR_PTR_TMP: *mut *mut c_void = 0 as _;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "cdecl" fn create_instance(
    _unk: *mut *mut c_void,
    character: *mut *mut c_void,
) -> bool {
    if DONE == 3 {
        return false;
    }
    DONE += 1;

    ACTOR_PTR_TMP = character;

    dbg!(ACTOR_PTR_TMP);

    false
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "cdecl" fn proc_dmg_and_stamina(
    actor: *mut *mut c_void,
    _param_2: u8,
    param_3: u32,
    dmg_mult: f32,
    stam_mult: f32,
) -> bool {
    println!(
        "proc_dmg_and_stamina: actor: {:p}, *actor: {:p} cached_actor: {:p}",
        actor, *actor, ACTOR_PTR_TMP
    );

    if actor == ACTOR_PTR_TMP {
        println!(
            "\tSuccess: param-3: {:p}, *param-3: {:p} dmg_mult: {}, stam_mult: {}",
            param_3 as *const c_void,
            *(param_3 as *const *const c_void),
            dmg_mult,
            stam_mult
        );
        return true;
    }
    false
}

#[no_mangle]
unsafe extern "stdcall" fn DllMain(
    _hinst_dll: HINSTANCE,
    fwd_reason: DWORD,
    _lpv_reserved: LPVOID,
) -> BOOL {
    if DetourIsHelperProcess() != 0 {
        return 1;
    }

    if fwd_reason == DLL_PROCESS_ATTACH {
        AllocConsole();
        println!("[EMF DllMain] DllMain Loaded");

        dbg!(SetProcessDEPPolicy(0));

        let info = PE32::get_module_information().optional_header();
        let page_start = info.ImageBase as HANDLE;
        let page_size = info.SizeOfImage as usize;

        remap_view_of_section(page_start, page_size, PAGE_EXECUTE_READWRITE).unwrap();

        DetourRestoreAfterWith();
        DetourTransactionBegin();
        DetourUpdateThread(GetCurrentThread() as _);

        let opt_headers = PE32::get_module_information().optional_header();
        ORIGINAL_START = (opt_headers.ImageBase + opt_headers.AddressOfEntryPoint) as _;
        DetourAttach(&mut ORIGINAL_START as _, main as _);

        println!("[EMF] Hooked Entrypoint");

        PROCDMGSTAM_WRAPPER = wrap_borland(&(proc_dmg_and_stamina as _), &PROCDMGSTAM_ORIG);

        DetourAttach(
            &mut PROCDMGSTAM_ORIG as _,
            PROCDMGSTAM_WRAPPER.as_ptr() as _,
        );

        CREATE_INSTANCE_WRAPPER = wrap_borland(&(create_instance as _), &CREATE_INSTANCE_ORIG);

        DetourAttach(
            &mut CREATE_INSTANCE_ORIG as _,
            CREATE_INSTANCE_WRAPPER.as_ptr() as _,
        );

        DetourTransactionCommit();
    }

    1
}

#[no_mangle]
unsafe extern "C" fn main() {
    println!("[EMF] Main Loaded");
    api::init_api();

    let original_start: extern "C" fn() = std::mem::transmute(ORIGINAL_START);
    original_start();
}
