// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use anyhow::Result;
use spz::{
	coord::CoordinateSystem,
	gaussian_splat::GaussianSplat,
	gaussian_splat::{LoadOptions, SaveOptions},
	packed::PackedGaussians,
};

fn main() -> Result<()> {
	let mut sample_spz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	sample_spz.push("assets/racoonfamily.spz");

	let gs = GaussianSplat::builder().load(sample_spz)?;

	let pg = gs.to_packed_gaussians(
		&SaveOptions::builder()
			.coord_sys(CoordinateSystem::RightUpBack) // packed will be in RUB (OpenGL)
			.build(),
	)?;
	let bytes = pg.as_bytes_vec()?;
	let pg2 = PackedGaussians::from_bytes(bytes.as_slice())?;
	let _gs2 = GaussianSplat::new_from_packed_gaussians(
		&pg2,
		&LoadOptions::builder()
			.coord_sys(CoordinateSystem::LeftUpFront) // _gs2 will be in LUF (glTF)
			.build(),
	)?;
	Ok(())
}
