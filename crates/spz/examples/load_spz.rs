// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use spz::{
	coord::CoordinateSystem, errors::SpzError, gaussian_splat::GaussianSplat,
	unpacked::UnpackOptions,
};

fn main() -> Result<(), SpzError> {
	let mut sample_spz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	sample_spz.push("assets/racoonfamily.spz");

	let _gs = GaussianSplat::builder()
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(CoordinateSystem::default())
				.build(),
		)
		.load(sample_spz)?;

	Ok(())
}

#[allow(unused)]
async fn load_spz_async<R>(reader: R) -> Result<GaussianSplat, SpzError>
where
	R: futures::io::AsyncRead + Unpin,
{
	GaussianSplat::builder()
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(CoordinateSystem::default())
				.build(),
		)
		.load_async(reader)
		.await
}
