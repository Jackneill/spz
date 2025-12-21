use std::path::PathBuf;

pub const SH_4BIT_EPSILON: f32 = 2.0 / 32.0 + 0.5 / 255.0;
pub const SH_5BIT_EPSILON: f32 = 2.0 / 64.0 + 0.5 / 255.0;

pub const EPSILON: f32 = 0.1;

pub fn assets_dir() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
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
