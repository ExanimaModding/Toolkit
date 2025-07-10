#![doc = include_str!("../README.md")]
#![feature(error_generic_member_access)]
#![feature(path_file_prefix)]
#![deny(missing_docs)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod gui;
mod injector;

use std::{env, path::Path, sync::OnceLock};

use clap::Parser;
use emcore::{Error, Result};
use tracing::instrument;
use tracing_subscriber::{EnvFilter, filter};

#[cfg(debug_assertions)]
#[global_allocator]
static GLOBAL: tracing_tracy::client::ProfiledAllocator<std::alloc::System> =
	tracing_tracy::client::ProfiledAllocator::new(std::alloc::System, 100);

/// When tracing is initialized for logging, the guard to the log file is stored
/// here to ensure tracing keeps writing to the log file.
pub(crate) static TRACING_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> =
	OnceLock::new();

#[instrument(level = "trace")]
fn main() {
	#[cfg(debug_assertions)]
	tracing_tracy::client::Client::start();

	if env::args_os().len() == 1 {
		let _ = gui::App::run().map_err(|e| eprintln!("failed to run gui: {e}"));
	} else {
		let body = async { cli::App::parse().run().await };
		tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.expect("Failed building the Runtime")
			.block_on(body);
	};
}

/// Helper function to parse a string into tracing's filter [`Directive`] for
/// logging.
#[instrument(level = "trace")]
fn directive(directive: &str) -> Option<filter::Directive> {
	directive
		.parse::<filter::Directive>()
		.map_err(|e| {
			eprintln!("failed to parse {directive} into directive for tracing filter: {e}")
		})
		.ok()
}

/// Helper function to add a filter to a tracing [`EnvFilter`] for logging.
#[instrument(level = "trace")]
fn add_directive(filter: EnvFilter, d: &str) -> EnvFilter {
	if let Some(directive) = directive(d) {
		filter.add_directive(directive)
	} else {
		filter
	}
}

/// Helper function to return a tracing [`EnvFilter`] for logging.
#[instrument(level = "trace")]
fn env_filter() -> EnvFilter {
	EnvFilter::builder()
		.from_env()
		.map_err(|e| eprintln!("failed to set filter from env, falling back to default: {e}"))
		.unwrap_or_default()
}

/// Executes Exanima as a process inside the given path's directory
#[instrument(level = "trace")]
pub fn launch(path: &Path) -> Result<()> {
	let exanima_exe = path
		.join(emcore::Instance::BINARY)
		.canonicalize()
		.map_err(Error::msg(
			"failed to find the game executable file in the game directory",
		))?;

	#[cfg(not(debug_assertions))]
	let emf_dll = env::current_exe()
		.map_err(Error::msg("failed to find path to current executable"))?
		.with_file_name("emf.dll")
		.display()
		.to_string();

	#[cfg(debug_assertions)]
	let emf_dll = env::current_exe()
		.unwrap()
		.parent()
		.unwrap()
		.join("deps")
		.join("emf.dll")
		.display()
		.to_string();

	unsafe {
		crate::injector::inject(&emf_dll, &exanima_exe.display().to_string())?;
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	#[test]
	fn cli() {}
}
