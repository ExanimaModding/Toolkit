mod framework;
mod internal;
mod plugins;

use std::{ffi::c_void, fs};

use detours_sys::{
	DetourAttach, DetourIsHelperProcess, DetourRestoreAfterWith, DetourTransactionBegin,
	DetourTransactionCommit,
};
use internal::{gui, utils::fs_redirector};
use pelite::pe::Pe;
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
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

		println!("DllMain Loaded");

		println!("Remapping Image");
		remap_image().unwrap();

		println!("Restoring Memory Import Table");
		DetourRestoreAfterWith();

		println!("Hooking Process Entrypoint");
		DetourTransactionBegin();
		let opt_headers = PE64::get_module_information().optional_header();
		ORIGINAL_START = (opt_headers.ImageBase + opt_headers.AddressOfEntryPoint as u64) as _;
		DetourAttach(&raw mut ORIGINAL_START, main as _);
		DetourTransactionCommit();
	}

	1
}

unsafe extern "C" fn main() {
	ansi_term::enable_ansi_support().unwrap();
	let log_dir = emcore::data_dir().join(emcore::LOG_DIR);
	if !log_dir.is_dir() {
		fs::create_dir_all(&log_dir).unwrap();
	}
	let file_appender = tracing_appender::rolling::hourly(log_dir, "emf.log");
	let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
	// initialize subscriber here instead of `DllMain` due to `WorkerGuard` being dropped
	tracing_subscriber::registry()
		.with(
			fmt::layer().with_filter(
				EnvFilter::builder()
					.from_env()
					.unwrap()
					.add_directive("emf=debug".parse().unwrap()),
			),
		)
		.with(
			fmt::layer()
				.with_ansi(false)
				.with_writer(non_blocking)
				.with_filter(
					EnvFilter::builder()
						.from_env()
						.unwrap()
						.add_directive("emf=debug".parse().unwrap()),
				),
		)
		.init();

	info!("Main Hook Running");

	// Redirect FS calls to the EMTK cache directory.
	fs_redirector::register_hooks();

	let mut curr_exe_path = std::env::current_exe().unwrap();
	curr_exe_path.pop();

	match std::env::set_current_dir(curr_exe_path) {
		Ok(_) => {}
		Err(e) => {
			error!(
				"Failed to set current directory: {:?}. This may cause unexpected behaviour.",
				e
			);
		}
	};

	// TODO: port plugin configs to emcore::plugin::Manifest
	// gui::inject_gui();

	// let plugin_configs = match read_plugin_configs() {
	// 	Ok(configs) => configs,
	// 	Err(e) => {
	// 		error!("Failed to load plugin configs: {:?}", e);
	// 		return;
	// 	}
	// };

	// for config in plugin_configs {
	// 	let result = plugins::load_plugin(config);
	// 	if let Err(e) = result {
	// 		error!("Failed to load plugin: {:?}", e);
	// 	}
	// }

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
