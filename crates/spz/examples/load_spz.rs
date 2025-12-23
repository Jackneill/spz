// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::{Path, PathBuf};

use anyhow::Result;
use spz::{GaussianSplat, UnpackOptions};

fn main() -> Result<()> {
	let mut sample_spz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	sample_spz.push("assets/racoonfamily.spz");

	let _gs = spz::GaussianSplat::builder()
		.filepath(sample_spz)
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(spz::CoordinateSystem::default())
				.build(),
		)
		.load()?;

	Ok(())
}

#[allow(unused)]
async fn load_spz_async<P>(spz_file: P) -> Result<GaussianSplat>
where
	P: AsRef<Path>,
{
	let gs = spz::GaussianSplat::builder()
		.filepath(spz_file)
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(spz::CoordinateSystem::default())
				.build(),
		)
		.load_async()
		.await?;

	Ok(gs)
}
