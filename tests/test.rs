// SPDX-License-Identifier: Apache-2.0 OR MIT

use approx::assert_relative_eq;
use rstest::rstest;
use spz::{GaussianSplat, PackOptions, UnpackOptions, gaussian_splat};

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
	bounding_box_x: [-199.94, 200.06],
	bounding_box_y: [-199.151, 200.849],
	bounding_box_z: [-198.665, 201.336],
})]
#[case("racoonfamily.spz", util::SpzValues {
	num_points: 932560,
	bounding_box_x: [-281.78, 258.383],
	bounding_box_y: [-240_f32, 240_f32],
	bounding_box_z: [-240_f32, 240_f32],
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

	assert_relative_eq!(min_x, spz_values.bounding_box_x[0], epsilon = util::EPSILON);
	assert_relative_eq!(max_x, spz_values.bounding_box_x[1], epsilon = util::EPSILON);
	assert_relative_eq!(min_y, spz_values.bounding_box_y[0], epsilon = util::EPSILON);
	assert_relative_eq!(max_y, spz_values.bounding_box_y[1], epsilon = util::EPSILON);
	assert_relative_eq!(min_z, spz_values.bounding_box_z[0], epsilon = util::EPSILON);
	assert_relative_eq!(max_z, spz_values.bounding_box_z[1], epsilon = util::EPSILON);
}
