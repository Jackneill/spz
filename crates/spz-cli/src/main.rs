// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mimalloc::MiMalloc;

use spz::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// CLI for SPZ (Gaussian Splat) files.
#[derive(Parser, Debug)]
#[command(name = "spz")]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
	/// Display metadata information about an SPZ file, from details found
	/// in the header.
	///
	/// Provides efficient access to metadata without loading the entire
	/// file, useful for quick inspection.
	/// Parses/Loads only the 1st 512 bytes of the file.
	Metainfo {
		/// Path to the SPZ file.
		spz_path: PathBuf,
	},
	/// Display information about an SPZ file.
	Info {
		/// Path to the SPZ file.
		spz_path: PathBuf,
	},
}

fn main() -> Result<ExitCode> {
	match run() {
		Ok(()) => Ok(ExitCode::SUCCESS),
		Err(err) => {
			eprintln!("error: {err}");

			Ok(ExitCode::FAILURE)
		},
	}
}

fn run() -> Result<()> {
	let cli = Cli::parse();

	match cli.command {
		Commands::Metainfo { spz_path: file } => cmd_metainfo(&file),
		Commands::Info { spz_path: file } => cmd_info(&file),
	}
}

fn cmd_info<P>(spz_path: P) -> Result<()>
where
	P: AsRef<Path>,
{
	let gs = GaussianSplat::load(spz_path.as_ref())
		.with_context(|| format!("failed to load SPZ file: {:?}", spz_path.as_ref()))?;

	print!("{}", gs.pretty_fmt());

	Ok(())
}

fn cmd_metainfo<P>(spz_path: P) -> Result<()>
where
	P: AsRef<Path>,
{
	let gsh = Header::from_file(&spz_path)
		.with_context(|| format!("failed to load SPZ file: {:?}", spz_path.as_ref()))?;

	print!("{}", gsh.pretty_fmt());

	Ok(())
}
