// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use clap::{Parser, Subcommand};
use repacker::{pack, types::rpk::RPK, unpack};
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
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Pack(args)) => {
            let src_path = PathBuf::from(&args.src);
            if src_path.is_dir() {
                let mut meta_path = src_path.clone();
                meta_path.push("metadata.toml");

                if meta_path.exists() {
                    RPK::pack(args.src.as_str(), args.dest.as_str())?;
                } else {
                    pack(args.src.as_str(), args.dest.as_str()).await?;
                }
            } else {
                eprintln!("Invalid path for source. Doing nothing");
            }
        }
        None => {
            let src_path = PathBuf::from(&cli.src);
            if src_path.is_file() {
                let mut dest_path = PathBuf::from(&cli.dest);
                dest_path.push(src_path.file_stem().unwrap());

                RPK::unpack(cli.src.as_str(), dest_path.to_str().unwrap()).await?;
            } else if src_path.is_dir() {
                unpack(cli.src.as_str(), cli.dest.as_str()).await?;
            } else {
                eprintln!("Invalid path for source. Doing nothing.");
            }
        }
    };

    Ok(())
}
