// SPDX-License-Identifier: Apache-2.0 OR MIT

use approx::assert_relative_eq;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rstest::rstest;
use spz::{CoordinateSystem, GaussianSplat, PackOptions, UnpackOptions, gaussian_splat};

use crate::util::mktmp;

mod util;

#[rstest]
#[case("hornedlizard.spz")]
#[case("racoonfamily.spz")]
fn test_load_packed_from_file_no_err(#[case] filename: &str) {
	let spz_path = util::assets_dir().join(filename);
	let unpack_opts = UnpackOptions::default();

	let _gs = GaussianSplat::load_packed_from_file(&spz_path, &unpack_opts)
		.expect("failed to load splat");
}

#[test]
fn test_empty_gaussian_splat() {
	let gs = GaussianSplat::default();

	assert_eq!(gs.num_points, 0);
	assert!(gs.positions.is_empty());
	assert!(gs.scales.is_empty());
	assert!(gs.rotations.is_empty());
	assert!(gs.alphas.is_empty());
	assert!(gs.colors.is_empty());
	assert!(gs.spherical_harmonics.is_empty());

	let packed_gs = gs.to_packed_gaussians(&PackOptions::default()).unwrap();

	assert!(packed_gs.positions.is_empty());
	assert!(packed_gs.scales.is_empty());
	assert!(packed_gs.rotations.is_empty());
	assert!(packed_gs.alphas.is_empty());
	assert!(packed_gs.colors.is_empty());
	assert!(packed_gs.spherical_harmonics.is_empty());

	assert!(GaussianSplat::load_packed(&[]).is_err());
}

#[rstest]
#[case("hornedlizard.spz")]
#[case("racoonfamily.spz")]
fn test_load_packed_bytes(#[case] filename: &str) {
	let spz_path = util::assets_dir().join(filename);
	let spz_infile = std::fs::read(&spz_path).expect("failed to read file");

	let pg = GaussianSplat::load_packed(&spz_infile).expect("failed to load packed");

	assert!(pg.num_points > 0);
	assert!(!pg.positions.is_empty());
	assert!(!pg.scales.is_empty());
	assert!(!pg.rotations.is_empty());
	assert!(!pg.alphas.is_empty());
	assert!(!pg.colors.is_empty());
	assert!(!pg.spherical_harmonics.is_empty());
}

#[rstest]
#[case("hornedlizard.spz")]
#[case("racoonfamily.spz")]
fn test_roundtrip_pack_unpack(#[case] filename: &str) {
	let original_spz = util::assets_dir().join(filename);
	let unpack_opts = UnpackOptions::default();
	let pack_opts = PackOptions::default();

	// original
	let original_gs = GaussianSplat::load_packed_from_file(&original_spz, &unpack_opts)
		.expect("failed to load splat");

	// serialize to bytes
	let packed_bytes = original_gs
		.serialize_as_packed_bytes(&pack_opts)
		.expect("failed to serialize");

	// reload from bytes
	let reloaded_fs_from_packed =
		GaussianSplat::load_packed(&packed_bytes).expect("failed to reload packed");
	let reloaded =
		GaussianSplat::new_from_packed_gaussians(&reloaded_fs_from_packed, &unpack_opts)
			.expect("failed to unpack");

	assert_eq!(original_gs.num_points, reloaded.num_points);
	assert_eq!(
		original_gs.spherical_harmonics_degree,
		reloaded.spherical_harmonics_degree
	);
	assert_eq!(original_gs.antialiased, reloaded.antialiased);

	assert_eq!(original_gs.positions.len(), reloaded.positions.len());
	assert_eq!(original_gs.scales.len(), reloaded.scales.len());
	assert_eq!(original_gs.rotations.len(), reloaded.rotations.len());
	assert_eq!(original_gs.alphas.len(), reloaded.alphas.len());
	assert_eq!(original_gs.colors.len(), reloaded.colors.len());
	assert_eq!(
		original_gs.spherical_harmonics.len(),
		reloaded.spherical_harmonics.len()
	);
	for (orig, reload) in original_gs.positions.iter().zip(reloaded.positions.iter()) {
		assert_relative_eq!(orig, reload, epsilon = util::EPSILON, max_relative = 0.01);
	}
}

#[rstest]
#[case("hornedlizard.spz", util::SpzValues {
	num_points: 786233,
	bbox_x: [-199.94, 200.06],
	bbox_y: [-199.151, 200.849],
	bbox_z: [-198.665, 201.336],
})]
#[case("racoonfamily.spz", util::SpzValues {
	num_points: 932560,
	bbox_x: [-281.78, 258.383],
	bbox_y: [-240_f32, 240_f32],
	bbox_z: [-240_f32, 240_f32],
})]
fn test_spz_values(#[case] filename: &str, #[case] spz_values: util::SpzValues) {
	let spz_path = util::assets_dir().join(filename);

	let gs = GaussianSplat::builder()
		.filepath(spz_path)
		.load()
		.expect("failed to load gaussian splat");

	assert_eq!(gs.num_points, spz_values.num_points);

	let gaussian_splat::BoundingBox {
		min_x,
		max_x,
		min_y,
		max_y,
		min_z,
		max_z,
	} = gs.bbox();

	assert_relative_eq!(min_x, spz_values.bbox_x[0], epsilon = util::EPSILON);
	assert_relative_eq!(max_x, spz_values.bbox_x[1], epsilon = util::EPSILON);
	assert_relative_eq!(min_y, spz_values.bbox_y[0], epsilon = util::EPSILON);
	assert_relative_eq!(max_y, spz_values.bbox_y[1], epsilon = util::EPSILON);
	assert_relative_eq!(min_z, spz_values.bbox_z[0], epsilon = util::EPSILON);
	assert_relative_eq!(max_z, spz_values.bbox_z[1], epsilon = util::EPSILON);
}

#[test]
fn test_save_load_packed_format_large_splat() {
	let num_points = 50_000_i32;
	let sh_deg = 3_i32;
	let sh_dim = 15_usize;

	let mut rng = StdRng::seed_from_u64(1);

	let positions: Vec<f32> = (0..(num_points as usize * 3))
		.map(|_| rng.random::<f32>() * 2.0 - 1.0)
		.collect();
	let scales: Vec<f32> = (0..(num_points as usize * 3))
		.map(|_| rng.random::<f32>() - 1.0)
		.collect();
	let rotations: Vec<f32> = (0..(num_points as usize * 4))
		.map(|_| rng.random::<f32>() * 2.0 - 1.0)
		.collect();
	let colors: Vec<f32> = (0..(num_points as usize * 3))
		.map(|_| rng.random::<f32>())
		.collect();
	let alphas: Vec<f32> = (0..num_points as usize)
		.map(|_| rng.random::<f32>())
		.collect();
	let spherical_harmonics: Vec<f32> = (0..(num_points as usize * sh_dim * 3))
		.map(|_| rng.random::<f32>() - 0.5)
		.collect();

	let src = GaussianSplat {
		num_points,
		spherical_harmonics_degree: sh_deg,
		antialiased: false,
		positions: positions.clone(),
		scales: scales.clone(),
		rotations: rotations.clone(),
		alphas: alphas.clone(),
		colors: colors.clone(),
		spherical_harmonics: spherical_harmonics.clone(),
	};
	let temp_dir = mktmp();

	let filename = temp_dir.join("large_splat.spz");
	let pack_opts = PackOptions::default();
	let unpack_opts = UnpackOptions::default();

	let packed_bytes = src
		.serialize_as_packed_bytes(&pack_opts)
		.expect("failed to serialize splat");

	std::fs::write(&filename, &packed_bytes).expect("failed to write file");

	let dst = GaussianSplat::load_packed_from_file(&filename, &unpack_opts)
		.expect("failed to load splat");

	assert_eq!(dst.num_points, src.num_points);
	assert_eq!(
		dst.spherical_harmonics_degree,
		src.spherical_harmonics_degree
	);
	for (orig, loaded) in src.positions.iter().zip(dst.positions.iter()) {
		assert_relative_eq!(orig, loaded, epsilon = 1.0 / 2048.0);
	}
	for (orig, loaded) in src.scales.iter().zip(dst.scales.iter()) {
		assert_relative_eq!(orig, loaded, epsilon = 1.0 / 16.0);
	}
	assert_eq!(dst.rotations.len(), src.rotations.len());

	for (orig, loaded) in src.alphas.iter().zip(dst.alphas.iter()) {
		assert_relative_eq!(orig, loaded, epsilon = 0.01);
	}
	let sh_epsilon = 0.15;

	for (orig, loaded) in src
		.spherical_harmonics
		.iter()
		.zip(dst.spherical_harmonics.iter())
	{
		assert_relative_eq!(orig, loaded, epsilon = sh_epsilon);
	}
	let _ = std::fs::remove_file(&filename);
}

#[test]
fn test_sh_encoding_for_zeros_and_edges() {
	let src = GaussianSplat {
		num_points: 1,
		spherical_harmonics_degree: 1,
		antialiased: false,
		positions: vec![0.0, 0.0, 0.0],
		scales: vec![0.0, 0.0, 0.0],
		rotations: vec![0.0, 0.0, 0.0, 1.0],
		alphas: vec![0.0],
		colors: vec![0.0, 0.0, 0.0],
		spherical_harmonics: vec![-0.01, 0.0, 0.01, -1.0, -0.99, -0.95, 0.95, 0.99, 1.0],
	};
	let temp_dir = mktmp();

	let filename = temp_dir.join("test_sh_encoding_for_zeros_and_edges.spz");

	let pack_opts = PackOptions::default();
	let unpack_opts = UnpackOptions::default();

	let packed_bytes = src
		.serialize_as_packed_bytes(&pack_opts)
		.expect("failed to serialize splat");
	std::fs::write(&filename, &packed_bytes).expect("failed to write file");

	let dst = GaussianSplat::load_packed_from_file(&filename, &unpack_opts)
		.expect("failed to load splat");

	assert_eq!(dst.num_points, 1);
	assert_eq!(dst.spherical_harmonics_degree, 1);

	let expected_sh: [f32; 9] = [
		-0.0625, 0.0, 0.0, -1.0, -1.0, -1.0, 0.9375, 0.9375, 0.9921875,
	];
	for (loaded, expected) in dst.spherical_harmonics.iter().zip(expected_sh.iter()) {
		assert_relative_eq!(loaded, expected, epsilon = 2e-5);
	}
	let _ = std::fs::remove_file(&filename);
}

#[test]
fn test_spherical_harmonics_coordinate_transformation() {
	// Set up test data with non-zero SH coefficients
	let original_sh: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];

	let src = GaussianSplat {
		num_points: 1,
		spherical_harmonics_degree: 1,
		antialiased: false,
		positions: vec![1.0, 2.0, 3.0],
		scales: vec![0.1, 0.2, 0.3],
		rotations: vec![0.0, 0.0, 0.0, 1.0],
		alphas: vec![0.5],
		colors: vec![0.1, 0.2, 0.3],
		spherical_harmonics: original_sh.clone(),
	};
	let temp_dir = mktmp();
	let filename = temp_dir.join("sh_coord_test.spz");

	// Save as RUB and load as RDF (180 degree rotation about X)
	let pack_opts = PackOptions {
		from: CoordinateSystem::RUB,
	};
	let packed_bytes = src
		.serialize_as_packed_bytes(&pack_opts)
		.expect("failed to serialize splat");
	std::fs::write(&filename, &packed_bytes).expect("failed to write file");

	let unpack_opts = UnpackOptions {
		to_coord_sys: CoordinateSystem::RDF,
	};
	let loaded = GaussianSplat::load_packed_from_file(&filename, &unpack_opts)
		.expect("failed to load splat");

	// Verify basic properties
	assert_eq!(loaded.num_points, 1);
	assert_eq!(loaded.spherical_harmonics_degree, 1);
	assert_eq!(loaded.spherical_harmonics.len(), 9);

	// The SH coefficients should be different from the original (transformed)
	// Some coefficients should be flipped according to the coordinate system change
	assert_ne!(loaded.spherical_harmonics, original_sh);

	// Verify the transformation is consistent - save and load again should give same result
	let filename2 = temp_dir.join("sh_coord_test2.spz");
	let pack_opts2 = PackOptions {
		from: CoordinateSystem::RDF,
	};
	let packed_bytes2 = loaded
		.serialize_as_packed_bytes(&pack_opts2)
		.expect("failed to serialize splat");
	std::fs::write(&filename2, &packed_bytes2).expect("failed to write file");

	let unpack_opts2 = UnpackOptions {
		to_coord_sys: CoordinateSystem::RDF,
	};
	let loaded2 = GaussianSplat::load_packed_from_file(&filename2, &unpack_opts2)
		.expect("failed to load splat");

	for (a, b) in loaded
		.spherical_harmonics
		.iter()
		.zip(loaded2.spherical_harmonics.iter())
	{
		// Use larger epsilon due to quantization loss in each save/load cycle
		assert_relative_eq!(a, b, epsilon = 0.1);
	}
	let _ = std::fs::remove_file(&filename);
	let _ = std::fs::remove_file(&filename2);
}
