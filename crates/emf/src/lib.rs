#![feature(raw_ref_op)]

mod framework;
mod internal;
mod plugins;

use log::*;

use std::ffi::c_void;

use detours_sys::{
	DetourAttach, DetourIsHelperProcess, DetourRestoreAfterWith, DetourTransactionBegin,
	DetourTransactionCommit,
};
use pelite::pe::Pe;
use winapi::{
	shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
	um::{consoleapi::AllocConsole, winnt::DLL_PROCESS_ATTACH},
};

use crate::{
	internal::utils::{pe64::PE64, remap_image},
	plugins::read_plugin_configs,
};

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

		pretty_env_logger::formatted_builder()
			.filter_level(LevelFilter::Trace)
			.init();

		info!("DllMain Loaded");

		info!("Remapping Image");
		remap_image().unwrap();

		info!("Restoring Memory Import Table");
		DetourRestoreAfterWith();

		info!("Hooking Process Entrypoint");
		DetourTransactionBegin();
		let opt_headers = PE64::get_module_information().optional_header();
		ORIGINAL_START = (opt_headers.ImageBase + opt_headers.AddressOfEntryPoint as u64) as _;
		DetourAttach(&raw mut ORIGINAL_START, main as _);
		DetourTransactionCommit();
	}

	1
}

unsafe extern "C" fn main() {
	info!("Main Hook Running");

	// if let Err(e) = init_dll_plugins() {
	// 	error!("[EMF] Failed to load DLL plugins: {:?}", e);
	// }

	let plugin_configs = match read_plugin_configs() {
		Ok(configs) => configs,
		Err(e) => {
			error!("Failed to load plugin configs: {:?}", e);
			return;
		}
	};

	for config in plugin_configs {
		let result = plugins::load_plugin(config);
		if let Err(e) = result {
			error!("Failed to load plugin: {:?}", e);
		}
	}

	info!("Running Original Program Entrypoint");

	// TODO: replace this with the new hooking system.
	let original_start: extern "C" fn() = std::mem::transmute(ORIGINAL_START);
	original_start();
}

// The following function is only necessary for the header generation.
#[cfg(feature = "headers")] // c.f. the `Cargo.toml` section
pub fn generate_headers() -> ::std::io::Result<()> {
	::safer_ffi::headers::builder().to_file("emf.h")?.generate()
}
