// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use rand::{Rng, SeedableRng, rngs::StdRng};
use spz::{coord::CoordinateSystem, gaussian_splat::GaussianSplat, math, unpacked::UnpackOptions};

pub fn create_splat(num_points: i32) -> GaussianSplat {
	let sh_degree = 2_i32;
	let sh_dim = math::dim_for_degree(sh_degree as u8);

	let mut rng = StdRng::seed_from_u64(42);

	let positions: Vec<f32> = (0..(num_points as usize * 3))
		.map(|_| rng.random::<f32>() * 2.0 - 1.0)
		.collect();
	let scales: Vec<f32> = (0..(num_points as usize * 3))
		.map(|_| rng.random::<f32>() * 4.0 - 2.0)
		.collect();
	let rotations: Vec<f32> = (0..(num_points as usize * 4))
		.map(|_| rng.random::<f32>() * 2.0 - 1.0)
		.collect();
	let alphas: Vec<f32> = (0..num_points as usize)
		.map(|_| rng.random::<f32>())
		.collect();
	let colors: Vec<f32> = (0..(num_points as usize * 3))
		.map(|_| rng.random::<f32>())
		.collect();
	let spherical_harmonics: Vec<f32> = (0..(num_points as usize * sh_dim as usize * 3))
		.map(|_| rng.random::<f32>() - 0.5)
		.collect();

	GaussianSplat {
		num_points,
		spherical_harmonics_degree: sh_degree,
		antialiased: false,
		positions,
		scales,
		rotations,
		alphas,
		colors,
		spherical_harmonics,
	}
}

pub fn load_packed_from_file() -> Result<GaussianSplat> {
	GaussianSplat::builder()
		.filepath("../../assets/racoonfamily.spz")
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(CoordinateSystem::default())
				.build(),
		)
		.load()
}

pub async fn load_packed_from_file_async() -> Result<GaussianSplat> {
	GaussianSplat::builder()
		.filepath("../../assets/racoonfamily.spz")
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(CoordinateSystem::default())
				.build(),
		)
		.load_async()
		.await
}
