#![doc = include_str!("../README.md")]
#![feature(let_chains)]
#![feature(error_generic_member_access)]
#![feature(path_file_prefix)]
#![deny(missing_docs)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod gui;
mod injector;

use std::{env, path::Path};

use clap::Parser;
use emcore::{Error, Result};
use tracing::instrument;

#[cfg(debug_assertions)]
#[global_allocator]
static GLOBAL: tracing_tracy::client::ProfiledAllocator<std::alloc::System> =
	tracing_tracy::client::ProfiledAllocator::new(std::alloc::System, 100);

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
