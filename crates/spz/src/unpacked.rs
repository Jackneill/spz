// SPDX-License-Identifier: Apache-2.0 OR MIT

use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};

use crate::coord::CoordinateSystem;

#[derive(Clone, Debug, Arbitrary)]
pub struct UnpackOptionsBuilder {
	to_coord_sys: CoordinateSystem,
}

impl UnpackOptionsBuilder {
	#[inline]
	pub fn to_coord_system(mut self, coord_sys: CoordinateSystem) -> Self {
		self.to_coord_sys = coord_sys;
		self
	}

	#[inline]
	pub fn build(self) -> UnpackOptions {
		UnpackOptions {
			to_coord_sys: self.to_coord_sys,
		}
	}
}

impl Default for UnpackOptionsBuilder {
	#[inline]
	fn default() -> Self {
		Self {
			to_coord_sys: CoordinateSystem::UNSPECIFIED,
		}
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Arbitrary)]
pub struct UnpackOptions {
	pub to_coord_sys: CoordinateSystem,
}

impl UnpackOptions {
	#[inline]
	pub fn builder() -> UnpackOptionsBuilder {
		UnpackOptionsBuilder::default()
	}
}

static_assertions::const_assert_eq!(std::mem::size_of::<UnpackedGaussian>(), 236);

/// Represents a single inflated gaussian.
///
/// Each gaussian has 236 bytes.
/// Although the data is easier to intepret in this format,
/// it is not more precise than the packed format, since it was inflated.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct UnpackedGaussian {
	pub position: [f32; 3], // x, y, z
	pub rotation: [f32; 4], // x, y, z, w
	pub scale: [f32; 3],    // std::log(scale)
	pub color: [f32; 3],    // rgb sh0 encoding
	pub alpha: f32,         // inverse logistic
	pub sh_r: [f32; 15],
	pub sh_g: [f32; 15],
	pub sh_b: [f32; 15],
}
