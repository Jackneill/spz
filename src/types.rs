use crate::consts;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GaussianCloudData {
	pub num_points: usize,
	pub sh_degree: usize,
	pub antialiased: bool,
	pub positions: Vec<f64>,
	pub scales: Vec<f64>,
	pub rotations: Vec<f64>,
	pub alphas: Vec<f64>,
	pub colors: Vec<f64>,
	pub sh: Vec<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct PackOptions {
	pub from: CoordinateSystem,
}

#[derive(Clone, Debug, Default)]
pub struct UnpackOptions {
	pub to: CoordinateSystem,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CoordinateSystem {
	#[default]
	UNSPECIFIED = 0,

	LDB = 1, // Left Down Back
	RDB = 2, // Right Down Back
	LUB = 3, // Left Up Back
	RUB = 4, // Right Up Back, Three.js coordinate system
	LDF = 5, // Left Down Front
	RDF = 6, // Right Down Front, PLY coordinate system
	LUF = 7, // Left Up Front, GLB coordinate system
	RUF = 8, // Right Up Front, Unity coordinate system
}

impl CoordinateSystem {
	pub fn axes_match(&self, other: crate::types::CoordinateSystem) -> (bool, bool, bool) {
		let self_num = self.clone() as usize - 1;
		let other_num = (other as usize) - 1;

		let xm = ((self_num >> 0) & 1) == ((other_num >> 0) & 1);
		let ym = ((self_num >> 1) & 1) == ((other_num >> 1) & 1);
		let zm = ((self_num >> 2) & 1) == ((other_num >> 2) & 1);

		(xm, ym, zm)
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoordinateConverter {
	pub from: CoordinateSystem,
	pub to: CoordinateSystem,
}

// Build local flip structs (avoids colliding with crate::types::CoordinateConverter).
pub struct CoordFlip {
	pub flip_p: [f64; 3],
	pub flip_q: [f64; 3],
	pub flip_sh: [f64; 15],
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PackedGaussiansHeader {
	/// NGSP = Niantic gaussian splat.
	pub magic: u32,
	pub version: u32,
	pub num_points: u32,
	pub sh_degree: u8,
	pub fractional_bits: u8,
	pub flags: u8,
	pub reserved: u8,
}

impl Default for PackedGaussiansHeader {
	fn default() -> Self {
		Self {
			magic: consts::PACKED_GAUSSIAN_HEADER_MAGIC,
			version: consts::PACKED_GAUSSIAN_HEADER_VERSION,
			num_points: 0,
			sh_degree: 0,
			fractional_bits: 0,
			flags: 0,
			reserved: 0,
		}
	}
}

impl From<[u8; 16]> for PackedGaussiansHeader {
	fn from(from: [u8; 16]) -> Self {
		unsafe { std::mem::transmute::<[u8; 16], Self>(from) }
	}
}
