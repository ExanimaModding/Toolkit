// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

mod api;
mod memory;
mod utils;

use libmem::lm_address_t;
use winapi::{
    shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
    um::{
        consoleapi::AllocConsole,
        winnt::{DLL_PROCESS_ATTACH, PAGE_EXECUTE_READWRITE},
    },
};

use crate::utils::pe32::PE32;
use detour::{Function, GenericDetour};

struct Hook<T: Function>(pub Option<GenericDetour<T>>);

impl<T: Function> Hook<T> {
    unsafe fn set(&mut self, detour: GenericDetour<T>) {
        self.0 = Some(detour);
    }
    unsafe fn enable(&mut self) -> Result<(), detour::Error> {
        self.0.as_mut().unwrap().enable()
    }

    unsafe fn disable(&mut self) -> Result<(), detour::Error> {
        self.0.as_mut().unwrap().disable()
    }
}

static mut ORIGINAL_START: lm_address_t = 0;
static mut ENTRYPOINT_HOOK: Hook<fn()> = Hook(None);

#[no_mangle]
extern "cdecl" fn ExportedFunction() {}

#[no_mangle]
unsafe extern "stdcall" fn DllMain(
    _hinst_dll: HINSTANCE,
    fwd_reason: DWORD,
    _lpv_reserved: LPVOID,
) -> BOOL {
    if fwd_reason != DLL_PROCESS_ATTACH {
        return 1;
    }

    AllocConsole();
    println!("[EMF] DllMain Loaded");

    utils::virtual_protect_module(PAGE_EXECUTE_READWRITE);
    println!("[EMF] VirtualProtect Done");

    ORIGINAL_START = PE32::get_entrypoint() as _;

    let original_start: fn() = std::mem::transmute(ORIGINAL_START);
    let entrypoint_hook = GenericDetour::<fn()>::new(original_start, entrypoint).unwrap();
    ENTRYPOINT_HOOK.set(entrypoint_hook);
    ENTRYPOINT_HOOK.enable().unwrap();

    println!("[EMF] Hooked Entrypoint");

    1
}

fn entrypoint() {
    unsafe {
        ENTRYPOINT_HOOK.disable().unwrap();
        main();
    };
}

unsafe fn main() {
    println!("[EMF] Main Loaded");
    api::init_api();
    println!("[EMF] API Initialized");

    std::thread::spawn(move || loop {
        if let Ok(health) = api::Player::get_stamina() {
            dbg!(health);
            api::Player::set_stamina(0.25).unwrap();
            api::Player::set_damage(0.25).unwrap();
        } else {
            println!("Stamina is none");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    });

    let original_start: extern "C" fn() = std::mem::transmute(ORIGINAL_START);
    original_start();
}
