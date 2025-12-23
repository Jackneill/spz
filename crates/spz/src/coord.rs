// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};

impl std::fmt::Display for CoordinateSystem {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CoordinateSystem::LDB => write!(f, "LDB"),
			CoordinateSystem::RDB => write!(f, "RDB"),
			CoordinateSystem::LUB => write!(f, "LUB"),
			CoordinateSystem::RUB => write!(f, "RUB"),
			CoordinateSystem::LDF => write!(f, "LDF"),
			CoordinateSystem::RDF => write!(f, "RDF"),
			CoordinateSystem::LUF => write!(f, "LUF"),
			CoordinateSystem::RUF => write!(f, "RUF"),
			_ => write!(f, "UNSPECIFIED"),
		}
	}
}

impl From<&str> for CoordinateSystem {
	#[inline]
	fn from(s: &str) -> Self {
		match s.to_uppercase().as_str() {
			"LDB" => CoordinateSystem::LDB,
			"RDB" => CoordinateSystem::RDB,
			"LUB" => CoordinateSystem::LUB,
			"RUB" => CoordinateSystem::RUB,
			"LDF" => CoordinateSystem::LDF,
			"RDF" => CoordinateSystem::RDF,
			"LUF" => CoordinateSystem::LUF,
			"RUF" => CoordinateSystem::RUF,
			_ => CoordinateSystem::UNSPECIFIED,
		}
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
	pub fn convert(&self, to: CoordinateSystem) -> CoordinateConverter {
		let (x_match, y_match, z_match) = self.axes_match(to);

		let x = if x_match { 1.0_f32 } else { -1.0_f32 };
		let y = if y_match { 1.0_f32 } else { -1.0_f32 };
		let z = if z_match { 1.0_f32 } else { -1.0_f32 };

		CoordinateConverter {
			flip_p: [x, y, z],
			flip_q: [y * z, x * z, x * y],
			flip_sh: [
				y,         // 0
				z,         // 1
				x,         // 2
				x * y,     // 3
				y * z,     // 4
				1.0_f32,   // 5
				x * z,     // 6
				1.0_f32,   // 7
				y,         // 8
				x * y * z, // 9
				y,         // 10
				z,         // 11
				x,         // 12
				z,         // 13
				x,         // 14
			],
		}
	}

	pub fn axes_match(&self, other: crate::coord::CoordinateSystem) -> (bool, bool, bool) {
		let self_num = self.clone() as i8 - 1;
		let other_num = (other as i8) - 1;

		if self_num < 0 || other_num < 0 {
			return (true, true, true);
		}
		let xm = ((self_num >> 0) & 1) == ((other_num >> 0) & 1);
		let ym = ((self_num >> 1) & 1) == ((other_num >> 1) & 1);
		let zm = ((self_num >> 2) & 1) == ((other_num >> 2) & 1);

		(xm, ym, zm)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CoordinateConverter {
	pub flip_p: [f32; 3],
	pub flip_q: [f32; 3],
	pub flip_sh: [f32; 15],
}

impl Default for CoordinateConverter {
	#[inline]
	fn default() -> Self {
		Self {
			flip_p: [1.0, 1.0, 1.0],
			flip_q: [1.0, 1.0, 1.0],
			flip_sh: [1.0; 15],
		}
	}
}
