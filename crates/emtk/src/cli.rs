//! The command line interface for the end-user to manage mods for their game
//! install

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use emcore::prelude::*;
use tokio::{
	fs,
	io::{self, AsyncReadExt},
};
// use termimad::crossterm::{
// 	self,
// 	terminal::{EnterAlternateScreen, LeaveAlternateScreen},
// };
use tracing::info;
// use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

/// The Exanima Modding CLI
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
pub(super) struct App {
	#[command(subcommand)]
	command: Option<AppCommands>,

	/// Disable printing logs for current command, useful for scripts
	#[arg(short, long)]
	silent: bool,

	/// Print more detailed logs. Overriden by --silent flag
	#[arg(short, long)]
	verbose: bool,
}

impl App {
	pub(super) async fn run(&self) {
		// let file_appender = tracing_appender::rolling::never(
		// 	std::path::absolute(PathBuf::from("./")).unwrap(),
		// 	"testing.log",
		// );
		// let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
		// let indicatif_layer = IndicatifLayer::new();

		if !self.silent {
			// TODO: enable indicatif in here
		}

		if self.verbose {
			tracing_subscriber::registry()
				.with(
					tracing_subscriber::fmt::layer()
						// .with_writer(indicatif_layer.get_stderr_writer())
						.with_filter(crate::env_filter()),
				)
				// .with(indicatif_layer)
				// .with(
				// 	tracing_subscriber::fmt::layer()
				// 		.with_writer(non_blocking)
				// 		.with_filter(crate::env_filter()),
				// )
				.init();
		}

		if let Some(cmd) = &self.command {
			cmd.run().await;
		}
	}
}

#[derive(Subcommand)]
enum AppCommands {
	/// Manage instances
	Instance {
		/// # Examples
		///
		/// ```sh
		/// $ emtk instance
		/// ```
		#[command(subcommand)]
		command: InstanceCommands,
	},
}

impl AppCommands {
	async fn run(&self) {
		match self {
			AppCommands::Instance { command } => command.run().await,
		}
	}
}

/// Manage instances
#[derive(Subcommand)]
enum InstanceCommands {
	Launch,
	/// Print out a history of imported instances
	List,
	/// Manage instance mods
	Mod {
		/// ```sh
		/// $ emtk instance mod
		/// ```
		#[command(subcommand)]
		command: ModCommands,
	},
	/// Manage instance profiles
	Profile {
		/// ```sh
		/// $ emtk instance profile
		/// ```
		#[command(subcommand)]
		command: ProfileCommands,
	},
	/// Import a game directory of Exanima as an instance
	Import {
		path: String,
	},
}

impl InstanceCommands {
	async fn run(&self) {
		match self {
			InstanceCommands::Launch => self.launch().await,
			InstanceCommands::List => self.list().await,
			InstanceCommands::Mod { command } => command.run().await,
			InstanceCommands::Profile { command } => command.run().await,
			InstanceCommands::Import { path } => self.import(path).await,
		}
	}

	async fn launch(&self) {
		let instance_history = instance::history().await.unwrap();
		let instance_path = instance_history.last().unwrap();
		let instance = Instance::with_path(instance_path)
			.unwrap()
			.build()
			.await
			.unwrap();
		crate::launch(instance.path().as_path()).unwrap();
	}

	/// Prints out the full path to a history of instances from the instance history
	/// file.
	///
	/// # Examples
	///
	/// ```sh
	/// $ emtk instance list
	/// ```
	async fn list(&self) {
		let instance_history = instance::history().await.unwrap();
		for path in instance_history.iter().rev() {
			println!("{}", path.to_str().unwrap());
		}
	}

	/// # Examples
	///
	/// ```sh
	/// $ emtk instance import "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Exanima"
	/// ```
	async fn import(&self, path: &str) {
		let path = PathBuf::from(path);
		Instance::with_path(path).unwrap().build().await.unwrap();
		info!("instance imported");
	}
}

/// Manage instance mods
#[derive(Subcommand)]
enum ModCommands {
	/// Print out README of a mod by their plugin ID
	Info { id: String },
	/// Print out currently installed mods for the instance
	List,
}

impl ModCommands {
	async fn run(&self) {
		match self {
			ModCommands::Info { id } => self.info(id).await,
			ModCommands::List => self.list().await,
		}
	}

	/// # Examples
	///
	/// ```sh
	/// emtk instance mod info com.example.my-mod
	/// ```
	async fn info(&self, maybe_id: &str) {
		let skin = termimad::MadSkin::default();
		let plugin_id = plugin::Id::try_from(maybe_id).unwrap();
		let instance_history = instance::history().await.unwrap();
		let instance_path = instance_history.last().unwrap();
		let instance = Instance::with_path(instance_path)
			.unwrap()
			.build()
			.await
			.unwrap();
		let mut read_mods_dir = fs::read_dir(instance.mods_dir().await.unwrap())
			.await
			.unwrap();
		while let Some(entry) = read_mods_dir.next_entry().await.unwrap() {
			let entry_path = entry.path();
			if !entry_path.is_dir() {
				continue;
			}
			if let Some(file_name_os) = entry_path.file_name()
				&& let Some(entry_name) = file_name_os.to_str()
				&& let Ok(entry_plugin_id) = plugin::Id::try_from(entry_name)
				&& entry_plugin_id == plugin_id
			{
				let readme_path = entry_path.join("README.md");
				if !readme_path.is_file() {
					break;
				}
				let file = fs::File::open(readme_path).await.unwrap();
				let mut reader = io::BufReader::new(file);
				let mut buffer = String::new();
				reader.read_to_string(&mut buffer).await.unwrap();

				// crossterm::execute!(io::stdout(), EnterAlternateScreen).unwrap();

				// skin.print_text(&buffer);
				skin.print_text(&skin.term_text(&buffer).to_string());

				// let _ = io::stdin().read(&mut []).unwrap();
				// crossterm::execute!(io::stdout(), LeaveAlternateScreen).unwrap();
				break;
			}
		}
	}

	/// # Examples
	///
	/// ```sh
	/// $ emtk instance mod list
	/// ```
	async fn list(&self) {
		let instance_history = instance::history().await.unwrap();
		let instance_path = instance_history.last().unwrap();
		let instance = Instance::with_path(instance_path)
			.unwrap()
			.build()
			.await
			.unwrap();
		let mut read_mods_dir = fs::read_dir(instance.mods_dir().await.unwrap())
			.await
			.unwrap();
		while let Some(entry) = read_mods_dir.next_entry().await.unwrap() {
			let entry_path = entry.path();
			if !entry_path.is_dir() {
				continue;
			};
			if let Some(file_name_os) = entry_path.file_name()
				&& let Some(entry_name) = file_name_os.to_str()
			{
				let Ok(plugin_id) = plugin::Id::try_from(entry_name) else {
					continue;
				};
				println!("{}", plugin_id);
			}
		}
	}
}

/// Manage instance profiles
#[derive(Subcommand)]
enum ProfileCommands {
	/// Create a new profile
	Create,
	// Delete an existing profile
	Delete {
		#[arg(short, long)]
		force: bool,
	},
	/// Print out the available profiles for the most recently imported instance
	List,
}

impl ProfileCommands {
	async fn run(&self) {
		match self {
			ProfileCommands::Create => self.create(),
			ProfileCommands::Delete { force } => self.delete(*force),
			ProfileCommands::List => self.list().await,
		}
	}

	fn create(&self) {
		todo!("profile create wip");
	}

	fn delete(&self, _force: bool) {
		todo!("profile delete wip");
	}

	async fn list(&self) {
		let instance_history = instance::history().await.unwrap();
		let instance_path = instance_history.last().unwrap();
		let instance = Instance::with_path(instance_path)
			.unwrap()
			.build()
			.await
			.unwrap();
		let profile_paths = instance.profile_dirs().await.unwrap();
		for path in profile_paths.iter().rev() {
			// TODO: if instance.profile.path == path, indicate profile as current?
			println!("{}", path.to_str().unwrap());
		}
	}
}
