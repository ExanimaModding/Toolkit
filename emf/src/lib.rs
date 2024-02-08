// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

mod framework;
mod internal;
mod mods;

use std::{ffi::c_void, ptr::addr_of_mut};

use detours_sys::{
	DetourAttach, DetourIsHelperProcess, DetourRestoreAfterWith, DetourTransactionBegin,
	DetourTransactionCommit,
};
use pelite::pe32::Pe;
use winapi::{
	shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
	um::{consoleapi::AllocConsole, winbase::SetProcessDEPPolicy, winnt::DLL_PROCESS_ATTACH},
};

use crate::internal::utils::{pe32::PE32, remap_image};

// TODO: Remove this when the new hooking system is implemented.
static mut ORIGINAL_START: *mut c_void = 0 as _;

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
		println!("[EMF DllMain] Disabling DEP Policy");
		SetProcessDEPPolicy(0);
		println!("[EMF DllMain] Remapping Image");
		remap_image().unwrap();
		println!("[EMF DllMain] Restoring Memory Import Table");
		DetourRestoreAfterWith();

		DetourTransactionBegin();
		let opt_headers = PE32::get_module_information().optional_header();
		ORIGINAL_START = (opt_headers.ImageBase + opt_headers.AddressOfEntryPoint) as _;
		DetourAttach(addr_of_mut!(ORIGINAL_START) as _, main as _);
		DetourTransactionCommit();
	}

	1
}

#[no_mangle]
unsafe extern "C" fn main() {
	println!("[EMF] Main Loaded");
	framework::api::init_api();

	// TODO: replace this with the new hooking system.
	let original_start: extern "C" fn() = std::mem::transmute(ORIGINAL_START);
	original_start();
}
