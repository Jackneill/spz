use std::path::PathBuf;

pub const SH_4BIT_EPSILON: f32 = 2.0 / 32.0 + 0.5 / 255.0;
pub const SH_5BIT_EPSILON: f32 = 2.0 / 64.0 + 0.5 / 255.0;

pub const EPSILON: f32 = 0.1;

pub fn assets_dir() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub struct SpzValues {
	pub num_points: i32,
	pub bounding_box_x: [f32; 2],
	pub bounding_box_y: [f32; 2],
	pub bounding_box_z: [f32; 2],
}
