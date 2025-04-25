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

#[cfg(debug_assertions)]
#[global_allocator]
static GLOBAL: tracing_tracy::client::ProfiledAllocator<std::alloc::System> =
	tracing_tracy::client::ProfiledAllocator::new(std::alloc::System, 100);

#[derive(Debug, thiserror::Error)]
enum Error {
	#[error("{message}")]
	Iced {
		message: &'static str,
		#[backtrace]
		source: iced::Error,
	},
}

/// General error types related to the crate is declared here
pub mod error {
	/// Failed to execute Exanima as a process
	#[derive(Debug, thiserror::Error)]
	pub enum Launch {
		/// Failed to complete a filesystem operation
		#[error("{0}")]
		Io(#[from] emcore::error::Io),
		/// Failed to resolve a path to a directory's parent
		#[error("{0}")]
		ParentDir(#[from] emcore::profile::error::ParentDir),
	}
}

fn main() -> Result<(), Error> {
	#[cfg(debug_assertions)]
	tracing_tracy::client::Client::start();

	if env::args_os().len() == 1 {
		return gui::App::run().map_err(|source| Error::Iced {
			message: "failed to run gui",
			source,
		});
	} else {
		let body = async { cli::App::parse().run().await };
		tokio::runtime::Builder::new_multi_thread()
			.enable_all()
			.build()
			.expect("Failed building the Runtime")
			.block_on(body);
	};

	Ok(())
}

/// tracing filter
pub fn env_filter() -> tracing_subscriber::EnvFilter {
	tracing_subscriber::EnvFilter::builder()
		.from_env()
		.unwrap()
		.add_directive("emtk=debug".parse().unwrap())
		.add_directive("emcore=debug".parse().unwrap())
}

/// Executes Exanima as a process inside the given path's directory
pub fn launch(path: &Path) -> Result<(), error::Launch> {
	let exanima_exe = path
		.join(emcore::Instance::BINARY)
		.canonicalize()
		.map_err(|source| emcore::error::Io {
			message: "failed to find the game executable file in the game directory",
			source,
		})?;

	#[cfg(not(debug_assertions))]
	let emf_dll = env::current_exe()
		.map_err(|source| emcore::error::Io {
			message: "failed to find path to current executable",
			source,
		})?
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
