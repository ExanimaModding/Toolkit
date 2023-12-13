// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

mod framework;
mod internal;

use detours_sys::{DetourIsHelperProcess, DetourRestoreAfterWith};
use pelite::pe32::Pe;
use winapi::{
	shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
	um::{consoleapi::AllocConsole, winbase::SetProcessDEPPolicy, winnt::DLL_PROCESS_ATTACH},
};

use crate::internal::{
	hooking::{
		hooks::{database::HookDB, detour::DetourHook, Hook, NewHook},
		HookName,
	},
	utils::{pe32::PE32, remap_image},
};

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

		let opt_headers = PE32::get_module_information().optional_header();

		let hook_name = HookName::internal("DllMain", "HookMain");

		let hook = Hook::new(
			hook_name.to_string(),
			DetourHook::new(
				(opt_headers.ImageBase + opt_headers.AddressOfEntryPoint) as usize,
				main as usize,
			),
		);

		HookDB.add_hook(hook).get_hook_mut().attach();
	}

	1
}

#[no_mangle]
unsafe extern "C" fn main() {
	println!("[EMF] Main Loaded");
	framework::api::init_api();

	let hook_name = HookName::internal("DllMain", "HookMain");
	let hook = HookDB.get_hook_mut(&hook_name.to_string()).unwrap();

	let main = hook.transmute::<extern "C" fn()>();
	main();
}
