// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use spz::{CoordinateSystem, UnpackOptions};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	Info {
		#[arg(help = "Path to the .spz file to read info from.")]
		spz_path: PathBuf,

		#[arg(
			default_value_t = CoordinateSystem::UNSPECIFIED,
			help = "Coordinate system to interpret the .spz file in."
		)]
		coordinate_system: CoordinateSystem,

		#[arg(
			short,
			long,
			help = "Output info as JSON to the given json file path instead of pretty printing to stdout."
		)]
		json: Option<PathBuf>,
	},
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	match &cli.command {
		Some(Commands::Info {
			spz_path: filepath,
			coordinate_system,
			json,
		}) => {
			if !filepath.exists() {
				bail!("error: file does not exist: {:?}", filepath);
			}
			if let Some(json_path) = json {
				output_json(filepath, coordinate_system.clone(), json_path)
					.with_context(|| "unable to output .spz info as JSON")?;
			} else {
				print_info(filepath, coordinate_system.clone())
					.with_context(|| "unable to print .spz info")?;
			}
		},
		None => {},
	}
	Ok(())
}

fn output_json<P>(spz_path: P, coord_sys: CoordinateSystem, output_file: P) -> Result<()>
where
	P: AsRef<Path> + Copy,
{
	let unpack_opts = UnpackOptions::builder()
		.to_coord_system(coord_sys.clone())
		.build();

	let gc = spz::GaussianSplat::builder()
		.filepath(spz_path)
		.packed(true)?
		.unpack_options(unpack_opts.clone())
		.load()?;

	let info = Info {
		num_points: gc.num_points,
		sh_degree: gc.spherical_harmonics_degree,
		antialiased: gc.antialiased,
		coordinate_system: unpack_opts.to_coord_sys,
		bbox: gc.bbox(),
		median_ellipsoid_volume: gc.median_volume(),
	};
	let json_output = serde_json::to_string_pretty(&info)
		.with_context(|| "unable to serialize .spz info as JSON")?;

	std::fs::write(output_file, json_output).with_context(|| {
		format!(
			"unable to write JSON output to file: {:?}",
			output_file.clone().as_ref()
		)
	})
}

fn print_info<P>(spz_path: P, coord_sys: CoordinateSystem) -> Result<()>
where
	P: AsRef<Path>,
{
	let unpack_opts = UnpackOptions::builder()
		.to_coord_system(coord_sys.clone())
		.build();

	let gc = spz::GaussianSplat::builder()
		.filepath(spz_path)
		.packed(true)?
		.unpack_options(unpack_opts.clone())
		.load()?;

	println!("{}", gc);

	Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Info {
	num_points: i32,
	sh_degree: i32,
	antialiased: bool,
	coordinate_system: CoordinateSystem,
	bbox: spz::gaussian_splat::BoundingBox,
	median_ellipsoid_volume: f32,
}
