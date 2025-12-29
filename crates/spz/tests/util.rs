// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

pub const SH_4BIT_EPSILON: f32 = 2.0 / 32.0 + 0.5 / 255.0;
pub const SH_5BIT_EPSILON: f32 = 2.0 / 64.0 + 0.5 / 255.0;

pub const EPSILON: f32 = 0.1;

pub fn assets_dir() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

pub fn mktmp() -> PathBuf {
	let tmp = assets_dir().parent().unwrap().join("target").join("tmp");

	std::fs::create_dir_all(&tmp).expect("failed to create temp dir");

	tmp
}

pub struct SpzValues {
	pub num_points: i32,
	pub bbox_x: [f32; 2],
	pub bbox_y: [f32; 2],
	pub bbox_z: [f32; 2],
}

pub fn quat_norm(q: &[f32]) -> f32 {
	(q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt()
}

pub fn normalize_quat(q: &[f32]) -> [f32; 4] {
	let n = quat_norm(q);

	[q[0] / n, q[1] / n, q[2] / n, q[3] / n]
}

pub fn quaternions_equivalent(q1: &[f32], q2: &[f32], tolerance: f32) -> bool {
	let same = q1
		.iter()
		.zip(q2.iter())
		.all(|(a, b)| (a - b).abs() < tolerance);

	let negated = q1
		.iter()
		.zip(q2.iter())
		.all(|(a, b)| (a + b).abs() < tolerance);

	same || negated
}
