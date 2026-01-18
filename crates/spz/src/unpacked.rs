// SPDX-License-Identifier: Apache-2.0 OR MIT

use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};

static_assertions::const_assert_eq!(std::mem::size_of::<UnpackedGaussian>(), 236);

/// Intermediate representation. Represents a single inflated gaussian.
///
/// Coordinate system conversions are already applied at this stage.
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
