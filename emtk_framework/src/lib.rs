mod internal;

use std::{
	collections::HashMap,
	env,
	ffi::c_void,
	fs,
	io::{self, Read, Write},
	mem,
	path::PathBuf,
	sync::OnceLock,
};

use detours_sys::{
	DetourAttach, DetourIsHelperProcess, DetourRestoreAfterWith, DetourTransactionBegin,
	DetourTransactionCommit,
};
use emtk_core::{plugin, profile};
use internal::utils::rpk_intercept;
use pelite::pe::Pe;
use tracing::{error, info, instrument};
use tracing_subscriber::{Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winapi::{
	shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
	um::{consoleapi::AllocConsole, winnt::DLL_PROCESS_ATTACH},
};

use crate::internal::utils::{pe64::PE64, remap_image};

#[cfg(debug_assertions)]
#[global_allocator]
static GLOBAL: tracing_tracy::client::ProfiledAllocator<std::alloc::System> =
	tracing_tracy::client::ProfiledAllocator::new(std::alloc::System, 100);

pub(crate) static LOAD_ORDER: OnceLock<Vec<(plugin::Id, profile::LoadOrderEntry)>> =
	OnceLock::new();
pub(crate) static MOD_ENTRIES: OnceLock<HashMap<String, HashMap<String, PathBuf>>> =
	OnceLock::new();

/// When tracing is initialized for logging, the guard to the log file is stored
/// here to ensure tracing keeps writing to the log file.
static TRACING_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

// TODO: Remove this when the new hooking system is implemented.
static mut ORIGINAL_START: *mut c_void = 0 as _;

#[unsafe(no_mangle)]
unsafe extern "stdcall" fn DllMain(
	_hinst_dll: HINSTANCE,
	fwd_reason: DWORD,
	_lpv_reserved: LPVOID,
) -> BOOL {
	unsafe {
		if DetourIsHelperProcess() != 0 {
			return 1;
		}

		if fwd_reason == DLL_PROCESS_ATTACH {
			AllocConsole();

			let mut lock = io::stdout().lock();
			let _ = writeln!(lock, "DllMain Loaded");

			let _ = writeln!(lock, "Remapping Image");
			remap_image().unwrap();

			let _ = writeln!(lock, "Restoring Memory Import Table");
			DetourRestoreAfterWith();

			let _ = writeln!(lock, "Hooking Process Entrypoint");
			DetourTransactionBegin();
			let opt_headers = PE64::get_module_information().optional_header();
			ORIGINAL_START = (opt_headers.ImageBase + opt_headers.AddressOfEntryPoint as u64) as _;
			DetourAttach(&raw mut ORIGINAL_START, main as _);
			DetourTransactionCommit();
		}
	}

	1
}

#[instrument(level = "trace")]
unsafe extern "C" fn main() {
	#[cfg(debug_assertions)]
	tracing_tracy::client::Client::start();

	let maybe_data_dir = emtk_core::data_dir();
	ansi_term::enable_ansi_support().unwrap();
	let subscriber = tracing_subscriber::registry().with(fmt::layer().with_filter(env_filter()));

	#[cfg(debug_assertions)]
	let subscriber = subscriber.with(tracing_tracy::TracyLayer::default());

	if let Some(data_dir) = maybe_data_dir {
		let log_dir = data_dir.join(emtk_core::LOG_DIR);
		if !log_dir.is_dir() {
			fs::create_dir_all(&log_dir).unwrap();
		}
		let file_appender = tracing_appender::rolling::hourly(log_dir, "emtk_framework.log");
		let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
		TRACING_GUARD.set(guard).unwrap();
		subscriber
			.with(
				fmt::layer()
					.with_ansi(false)
					.with_writer(non_blocking)
					.with_filter(env_filter()),
			)
			.init();
	} else {
		subscriber.init();
	}

	let mut cwd = env::current_exe().unwrap();
	cwd.pop();

	let load_order_path = env::var("EMTK_LOAD_ORDER_PATH")
		.map_err(|e| error!("{}", e))
		.expect("EMTK_LOAD_ORDER_PATH must be set");
	let mut file = fs::File::open(&load_order_path)
		.map_err(|e| error!("{}", e))
		.expect("load order file must be available on disk");
	let mut buffer = String::new();
	file.read_to_string(&mut buffer)
		.map_err(|e| error!("{}", e))
		.expect("load order contents must be valid UTF-8");
	let load_order: Vec<_> = toml::from_str::<profile::LoadOrder>(&buffer)
		.expect("load order contents must be valid toml and structure")
		.into_iter()
		.filter(|(id, entry)| {
			if !entry.enabled {
				return false;
			}
			let Ok(mut file) =
				fs::File::open(cwd.join(id.plugin_dir().join(plugin::Manifest::TOML)))
			else {
				return false;
			};
			let mut buffer = String::new();
			let Ok(_) = file.read_to_string(&mut buffer) else {
				return false;
			};
			let Ok(_) = toml::from_str::<plugin::Manifest>(&buffer) else {
				return false;
			};
			true
		})
		.collect();

	let native_packages: Vec<_> = cwd
		.read_dir()
		.expect("error while reading game directory")
		.flatten()
		.filter_map(|entry| {
			let path = entry.path();
			let file_name = path.display().to_string();
			if path.is_dir() || !file_name.ends_with(".rpk") {
				None
			} else {
				Some(path.file_stem().unwrap().display().to_string())
			}
		})
		.collect();
	let mut custom_packages: HashMap<String, HashMap<String, PathBuf>> = HashMap::new();
	for name in native_packages {
		let mut mod_entries: HashMap<String, PathBuf> = HashMap::new();
		for (id, _) in load_order.iter() {
			let loose_files_path = id.packages_dir().join(&name);
			if let Ok(foreign_dir) = fs::read_dir(cwd.join(loose_files_path)) {
				let entries: Vec<_> = foreign_dir.filter_map(Result::ok).collect();
				entries.iter().for_each(|entry| {
					mod_entries.insert(entry.file_name().display().to_string(), entry.path());
				});
			} else {
				continue;
			}
		}
		custom_packages.insert(name, mod_entries);
	}

	MOD_ENTRIES.set(custom_packages).unwrap();
	LOAD_ORDER.set(load_order).unwrap();

	info!("Main Hook Running");

	// Redirect FS calls to the EMTK cache directory.
	// fs_redirector::register_hooks();

	if !LOAD_ORDER.get().unwrap().is_empty() {
		unsafe { rpk_intercept::register_hooks() };
	}

	match env::set_current_dir(cwd) {
		Ok(_) => {}
		Err(e) => {
			error!(
				"Failed to set current directory: {:?}. This may cause unexpected behaviour.",
				e
			);
		}
	};

	// TODO: port plugin configs to emtk_core::plugin::Manifest
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
	let original_start: extern "C" fn() = unsafe { mem::transmute(ORIGINAL_START) };
	original_start();
}

// The following function is only necessary for the header generation.
#[cfg(feature = "headers")] // c.f. the `Cargo.toml` section
#[instrument(level = "trace")]
pub fn generate_headers() -> ::std::io::Result<()> {
	::safer_ffi::headers::builder().to_file("emtk_framework.h")?.generate()
}

/// tracing filter
#[instrument(level = "trace")]
pub fn env_filter() -> tracing_subscriber::EnvFilter {
	tracing_subscriber::EnvFilter::builder()
		.from_env()
		.unwrap()
		.add_directive("emtk_framework=debug".parse().unwrap())
}
