use clap::{Parser, Subcommand};
use log::*;
use repacker::{constants::METADATA_FILE, pack, types::rpk::RPK};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author = "ProffDea <deatea@riseup.net>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Repacker cli tool")]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,

	#[arg(short, long)]
	#[arg(default_value = ".")]
	#[arg(help = "Directory path to game file(s) to unpack")]
	src: String,

	#[arg(short, long)]
	#[arg(default_value = "./.unpacked")]
	#[arg(help = "Directory path to dump contents of unpacked file(s)")]
	dest: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
	#[command(about = "Convert target folder to game file(s)")]
	Pack(Args),
}

#[derive(Parser, Debug)]
struct Args {
	#[arg(short, long)]
	#[arg(default_value = "./.unpacked")]
	#[arg(help = "Directory path to pack contents of a folder to game file(s)")]
	src: String,

	#[arg(short, long)]
	#[arg(default_value = "./.packed")]
	#[arg(help = "Directory path to dump packed game file(s)")]
	dest: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	if std::env::var("RUST_LOG").is_err() {
		std::env::set_var("RUST_LOG", "info");
	};
	pretty_env_logger::init();

	let cli = Cli::parse();

	let exanima_exe = match std::env::var("EXANIMA_EXE") {
		Ok(var) => PathBuf::from(var),
		Err(_) => PathBuf::from("./Exanima.exe"),
	};
	std::env::set_current_dir(
		exanima_exe
			.parent()
			.expect("error trying to get parent folder of Exanima.exe"),
	)
	.expect("error while setting the current working directory to Exanima.exe");

	match cli.command {
		Some(Commands::Pack(args)) => {
			let src_path = PathBuf::from(&args.src);
			if src_path.is_dir() {
				let mut meta_path = src_path.clone();
				meta_path.push(METADATA_FILE);

				if meta_path.exists() {
					RPK::pack(args.src.as_str(), args.dest.as_str())?;
				} else {
					pack(args.src.as_str(), args.dest.as_str()).await?;
				}
			} else {
				error!("Invalid path for source. Doing nothing");
			}
		}
		None => {
			repacker::unpack(cli.src.as_str(), cli.dest.as_str()).await?;
		}
	};

	Ok(())
}
