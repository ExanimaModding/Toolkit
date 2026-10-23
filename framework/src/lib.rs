#![feature(thread_id_value)]

mod framework;
mod internal;
mod plugins;

use crate::internal::gui;
use crate::internal::utils::{pe64::PE64, remap_image};
use crate::plugins::read_plugin_configs;

use detours_sys::{
    DetourAttach, DetourIsHelperProcess, DetourRestoreAfterWith, DetourTransactionBegin,
    DetourTransactionCommit,
};
use emtk::{plugin, profile};
use internal::utils::rpk_intercept;
use log::{info, warn};
use pelite::pe::Pe;
use tracing_subscriber::{Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use windows::Win32::Foundation::{HINSTANCE, TRUE};
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxA};
use windows::core::{BOOL, PCSTR};

use std::collections::HashMap;
use std::ffi::{CString, c_void};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::{env, fs, mem, panic, thread};

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
unsafe extern "system" fn DllMain(
    _hmodule: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    unsafe {
        if DetourIsHelperProcess() != 0 {
            return TRUE;
        }

        if reason == DLL_PROCESS_ATTACH {
            panic_hook_setup();
            _ = AllocConsole();
            logging();
            info!("Dll loaded");

            info!("Remapping image");
            remap_image().expect("Remap image");

            info!("Restoring memory import table");
            DetourRestoreAfterWith();

            info!("Hooking process entrypoint");
            DetourTransactionBegin();
            let opt_headers = PE64::get_module_information().optional_header();
            ORIGINAL_START = (opt_headers.ImageBase + opt_headers.AddressOfEntryPoint as u64) as _;
            DetourAttach(&raw mut ORIGINAL_START, main as _);
            DetourTransactionCommit();
        }
    }

    TRUE
}

unsafe extern "C" fn main() {
    info!("Main hook running");

    if let Ok(cwd) =
        current_exe().map_err(|e| warn!("Failed getting current working directory: {e}"))
        && let Ok(load_order) =
            get_load_order(&cwd).map_err(|e| warn!("Failed getting load order: {e}"))
    {
        let custom_packages = get_custom_packages(cwd, &load_order);

        MOD_ENTRIES.set(custom_packages).expect("Set MOD_ENTRIES");
        LOAD_ORDER.set(load_order).expect("Set LOAD_ORDER");
    }

    // Redirect FS calls to the EMTK cache directory.
    // fs_redirector::register_hooks();

    if !LOAD_ORDER.get().is_some_and(Vec::is_empty) {
        unsafe { rpk_intercept::register_hooks() };
    }

    // TODO: port plugin configs to emtk::plugin::Manifest
    gui::inject_gui();

    if let Ok(plugin_configs) =
        read_plugin_configs().map_err(|e| warn!("Failed to load plugin configs: {e}"))
    {
        for config in plugin_configs {
            _ = plugins::load_plugin(config).map_err(|e| warn!("Failed to load plugin: {e:?}"))
        }
    }

    info!("Running original program entrypoint");

    // TODO: replace this with the new hooking system.
    let original_start: extern "C" fn() = unsafe { mem::transmute(ORIGINAL_START) };
    original_start();
}

// The following function is only necessary for the header generation.
#[cfg(feature = "headers")] // c.f. the `Cargo.toml` section
pub fn generate_headers() -> ::std::io::Result<()> {
    ::safer_ffi::headers::builder()
        .to_file("emtk_framework.h")?
        .generate()
}

/// tracing filter
pub fn env_filter() -> tracing_subscriber::EnvFilter {
    tracing_subscriber::EnvFilter::builder()
        .from_env()
        .unwrap()
        .add_directive("emtk_framework=debug".parse().unwrap())
}

fn get_load_order(
    instance: impl AsRef<Path>,
) -> io::Result<Vec<(plugin::Id, profile::LoadOrderEntry)>> {
    let instance = instance.as_ref();

    let load_order_path = env::var("EMTK_LOAD_ORDER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from_iter([
                emtk::Instance::DATA_DIR,
                emtk::Instance::PROFILES_DIR,
                emtk::Instance::DEFAULT_PROFILE_DIR,
                emtk::Profile::LOAD_ORDER_TOML,
            ])
        });
    let mut file = fs::File::open(&load_order_path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    let load_order = toml::from_str::<profile::LoadOrder>(&buffer)
        .map_err(io::Error::other)?
        .into_iter()
        .filter(|(id, entry)| {
            if !entry.enabled {
                return false;
            }
            let Ok(mut file) =
                fs::File::open(instance.join(id.plugin_dir().join(plugin::Manifest::TOML)))
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

    Ok(load_order)
}

fn get_native_packages(instance: impl AsRef<Path>) -> Vec<String> {
    let instance = instance.as_ref();

    instance
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
        .collect()
}

fn get_custom_packages(
    instance: impl AsRef<Path>,
    load_order: &[(plugin::Id, profile::LoadOrderEntry)],
) -> HashMap<String, HashMap<String, PathBuf>> {
    let instance = instance.as_ref();

    let native_packages = get_native_packages(instance);
    let mut custom_packages: HashMap<String, HashMap<String, PathBuf>> = HashMap::new();
    for name in native_packages {
        let mut mod_entries: HashMap<String, PathBuf> = HashMap::new();
        for (id, _) in load_order.iter() {
            let loose_files_path = id.packages_dir().join(&name);
            if let Ok(foreign_dir) = fs::read_dir(instance.join(loose_files_path)) {
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
    custom_packages
}

fn logging() {
    let maybe_data_dir = emtk::data_dir();
    ansi_term::enable_ansi_support().expect("Enable ANSI code support");
    let subscriber = tracing_subscriber::registry().with(fmt::layer().with_filter(env_filter()));

    if let Some(data_dir) = maybe_data_dir {
        let log_dir = data_dir.join(emtk::LOG_DIR);
        if !log_dir.is_dir() {
            fs::create_dir_all(&log_dir).expect("Create log directory");
        }
        let file_appender = tracing_appender::rolling::hourly(log_dir, "emtk_framework.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        TRACING_GUARD.set(guard).expect("Set TRACING_GUARD");
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
}

fn panic_hook_setup() {
    panic::set_hook(Box::new(move |info| {
        if !Path::new("logs").exists() {
            _ = fs::create_dir("logs");
        }

        let msg = info.payload_as_str();
        let location = info.location();
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/emf-panicked.log")
            .ok();

        if let Some(((msg, location), mut file)) = msg.zip(location).zip(file) {
            let current_thread = thread::current();
            let name = current_thread.name().unwrap_or("<unnamed>");
            let tid = current_thread.id().as_u64().get();

            let output = format!("\nthread '{name}' ({tid:?}) panicked at {location}:\n{msg}");

            _ = file.write_all(output.as_bytes()); // TODO append .2026-02-22-14 format for tracing-appender compat?

            unsafe {
                MessageBoxA(
                    None,
                    PCSTR(CString::new(output).unwrap().as_ptr().cast()),
                    PCSTR(
                        CString::new(format!(
                            "{}@v{}",
                            env!("CARGO_PKG_NAME"),
                            env!("CARGO_PKG_VERSION")
                        ))
                        .unwrap()
                        .as_ptr()
                        .cast(),
                    ),
                    MB_ICONERROR,
                );
            }
        }
    }));
}

fn current_exe() -> io::Result<PathBuf> {
    let mut cwd = env::current_exe()?;
    cwd.pop();
    env::set_current_dir(&cwd)?;
    Ok(cwd)
}
