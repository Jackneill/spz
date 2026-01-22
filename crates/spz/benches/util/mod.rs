// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use anyhow::Result;
use rand::{Rng, SeedableRng, rngs::StdRng};
use spz::{coord::CoordinateSystem, gaussian_splat::GaussianSplat, header::Header, math};

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
		header: Header {
			num_points,
			spherical_harmonics_degree: sh_degree as u8,
			..Default::default()
		},
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
		.packed(true)?
		.coord_sys(CoordinateSystem::default())
		.load("../../assets/racoonfamily.spz")
}

pub async fn load_packed_from_file_async() -> Result<GaussianSplat> {
	GaussianSplat::builder()
		.packed(true)?
		.coord_sys(CoordinateSystem::default())
		.load_async("../../assets/racoonfamily.spz")
		.await
}

pub fn tmpdir() -> Result<PathBuf> {
	let rand: u64 = rand::random();

	let mut temp_dir = std::env::temp_dir();
	temp_dir.push(format!("{}", rand));

	std::fs::create_dir_all(&temp_dir)?;

	Ok(temp_dir)
}
