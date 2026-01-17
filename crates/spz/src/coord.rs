// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::str::FromStr;

use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};

/// Supported 3D coordinate systems for Gaussian splat data.
///
/// To aid with coordinate system conversions, callers should specify the
/// coordinate system their Gaussian Splat data is represented in when saving
/// and what coordinate system their rendering system uses when loading.
///
/// These are specified in the [`PackOptions`] and [`UnpackOptions`] respectively.
/// If the coordinate system is `Unspecified`, data will be saved and loaded
/// without conversion, which may harm interoperability.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Arbitrary)]
pub enum CoordinateSystem {
	#[default]
	Unspecified = 0,

	LeftDownBack = 1,
	RightDownBack = 2,
	LeftUpBack = 3,
	RightUpBack = 4, // Three.js coordinate system
	LeftDownFront = 5,
	RightDownFront = 6, // PLY coordinate system
	LeftUpFront = 7,    // GLB coordinate system
	RightUpFront = 8,   // Unity coordinate system
}

impl std::fmt::Display for CoordinateSystem {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CoordinateSystem::LeftDownBack => write!(f, "Left-Down-Back"),
			CoordinateSystem::RightDownBack => write!(f, "Right-Down-Back"),
			CoordinateSystem::LeftUpBack => write!(f, "Left-Up-Back"),
			CoordinateSystem::RightUpBack => write!(f, "Right-Up-Back"),
			CoordinateSystem::LeftDownFront => write!(f, "Left-Down-Front"),
			CoordinateSystem::RightDownFront => write!(f, "Right-Down-Front"),
			CoordinateSystem::LeftUpFront => write!(f, "Left-Up-Front"),
			CoordinateSystem::RightUpFront => write!(f, "Right-Up-Front"),
			CoordinateSystem::Unspecified => write!(f, "Unspecified"),
		}
	}
}

impl FromStr for CoordinateSystem {
	type Err = ();

	#[inline]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_uppercase().as_str() {
			"LDB" | "LEFTDOWNBACK" | "LEFT-DOWN-BACK" | "LEFT_DOWN_BACK" => {
				Ok(CoordinateSystem::LeftDownBack)
			},
			"RDB" | "RIGHTDOWNBACK" | "RIGHT-DOWN-BACK" | "RIGHT_DOWN_BACK" => {
				Ok(CoordinateSystem::RightDownBack)
			},
			"LUB" | "LEFTUPBACK" | "LEFT-UP-BACK" | "LEFT_UP_BACK" => {
				Ok(CoordinateSystem::LeftUpBack)
			},
			"RUB" | "RIGHTUPBACK" | "RIGHT-UP-BACK" | "RIGHT_UP_BACK" => {
				Ok(CoordinateSystem::RightUpBack)
			},
			"LDF" | "LEFTDOWNFRONT" | "LEFT-DOWN-FRONT" | "LEFT_DOWN_FRONT" => {
				Ok(CoordinateSystem::LeftDownFront)
			},
			"RDF" | "RIGHTDOWNFRONT" | "RIGHT-DOWN-FRONT" | "RIGHT_DOWN_FRONT" => {
				Ok(CoordinateSystem::RightDownFront)
			},
			"LUF" | "LEFTUPFRONT" | "LEFT-UP-FRONT" | "LEFT_UP_FRONT" => {
				Ok(CoordinateSystem::LeftUpFront)
			},
			"RUF" | "RIGHTUPFRONT" | "RIGHT-UP-FRONT" | "RIGHT_UP_FRONT" => {
				Ok(CoordinateSystem::RightUpFront)
			},
			_ => Ok(CoordinateSystem::Unspecified),
		}
	}
}

impl From<&str> for CoordinateSystem {
	#[inline]
	fn from(s: &str) -> Self {
		s.parse().unwrap_or(CoordinateSystem::Unspecified)
	}
}

impl CoordinateSystem {
	/// Returns an iterator over all coordinate system variants.
	///
	/// Useful for enumerating supported systems in UI or validation.
	///
	/// # Example
	///
	/// ```
	/// use spz::CoordinateSystem;
	///
	/// for coord in CoordinateSystem::iter() {
	///     println!("{}", coord.as_short_str());
	/// }
	/// ```
	pub fn iter() -> impl Iterator<Item = CoordinateSystem> {
		[
			CoordinateSystem::Unspecified,
			CoordinateSystem::LeftDownBack,
			CoordinateSystem::RightDownBack,
			CoordinateSystem::LeftUpBack,
			CoordinateSystem::RightUpBack,
			CoordinateSystem::LeftDownFront,
			CoordinateSystem::RightDownFront,
			CoordinateSystem::LeftUpFront,
			CoordinateSystem::RightUpFront,
		]
		.into_iter()
	}

	/// Returns a short 3-letter abbreviation for the coordinate system.
	///
	/// The abbreviation encodes the axis directions:
	/// - First letter: `L` (Left) or `R` (Right) for X-axis
	/// - Second letter: `U` (Up) or `D` (Down) for Y-axis
	/// - Third letter: `F` (Front) or `B` (Back) for Z-axis
	///
	/// Returns `"UNSPECIFIED"` for [`CoordinateSystem::Unspecified`].
	///
	/// # Example
	///
	/// ```
	/// use spz::CoordinateSystem;
	///
	/// assert_eq!(CoordinateSystem::RightDownFront.as_short_str(), "RDF");
	/// assert_eq!(CoordinateSystem::LeftUpFront.as_short_str(), "LUF");
	/// ```
	pub fn as_short_str(&self) -> &'static str {
		match self {
			CoordinateSystem::LeftDownBack => "LDB",
			CoordinateSystem::RightDownBack => "RDB",
			CoordinateSystem::LeftUpBack => "LUB",
			CoordinateSystem::RightUpBack => "RUB",
			CoordinateSystem::LeftDownFront => "LDF",
			CoordinateSystem::RightDownFront => "RDF",
			CoordinateSystem::LeftUpFront => "LUF",
			CoordinateSystem::RightUpFront => "RUF",
			CoordinateSystem::Unspecified => "UNSPECIFIED",
		}
	}

	/// Computes the axis flip multipliers needed to convert from `self` to `target`.
	///
	/// Returns an [`AxisFlips`], containing sign multipliers (`1.0` or `-1.0`) for
	/// _positions_, _rotations_, and _spherical harmonics_.
	/// Multiply each component by its corresponding sign to transform data
	/// between coordinate systems.
	///
	/// # Example
	///
	/// ```
	/// use spz::CoordinateSystem;
	///
	/// // PLY uses RightDownFront, GLB uses LeftUpFront
	/// let axis_flips = CoordinateSystem::RightDownFront.axis_flips_to(CoordinateSystem::LeftUpFront);
	///
	/// // Apply to a position
	/// let ply_pos = [1.0, 2.0, 3.0];
	/// let glb_pos = [
	///     ply_pos[0] * axis_flips.position[0],
	///     ply_pos[1] * axis_flips.position[1],
	///     ply_pos[2] * axis_flips.position[2],
	/// ];
	/// ```
	pub fn axis_flips_to(self, target: CoordinateSystem) -> AxisFlips {
		let (x_match, y_match, z_match) = self.axes_align(target);

		let x = if x_match { 1.0_f32 } else { -1.0_f32 };
		let y = if y_match { 1.0_f32 } else { -1.0_f32 };
		let z = if z_match { 1.0_f32 } else { -1.0_f32 };

		AxisFlips {
			position: [x, y, z],
			rotation: [y * z, x * z, x * y],
			spherical_harmonics: [
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

	/// Compares axis orientations between two coordinate systems.
	///
	/// Returns a tuple of booleans `(x, y, z)` indicating whether each axis
	/// points in the same direction in both systems.
	///
	/// - `true` means the axes match (no flip needed)
	/// - `false` means they're opposite (flip needed)
	///
	/// If either system is [`CoordinateSystem::Unspecified`], all axes are
	/// treated as matching (returns `(true, true, true)`).
	///
	/// # Example
	///
	/// ```
	/// use spz::CoordinateSystem;
	///
	/// // RightUpBack vs LeftUpFront: X differs, Y matches, Z differs
	/// let (x, y, z) = CoordinateSystem::RightUpBack.axes_align(CoordinateSystem::LeftUpFront);
	///
	/// assert_eq!((x, y, z), (false, true, false));
	/// ```
	pub fn axes_align(self, other: CoordinateSystem) -> (bool, bool, bool) {
		let self_num = self as i8 - 1;
		let other_num = other as i8 - 1;

		if self_num < 0 || other_num < 0 {
			return (true, true, true);
		}
		let xm = ((self_num >> 0) & 1) == ((other_num >> 0) & 1);
		let ym = ((self_num >> 1) & 1) == ((other_num >> 1) & 1);
		let zm = ((self_num >> 2) & 1) == ((other_num >> 2) & 1);

		(xm, ym, zm)
	}
}

/// Sign multipliers (+1.0 or -1.0) for transforming Gaussian splat data between
/// coordinate systems.
///
/// When converting splats from one [`CoordinateSystem`] to another, each axis may need
/// to be flipped (negated) to account for differences in handedness or axis orientation.
/// This struct holds the sign values (`1.0` or `-1.0`) to multiply against the respective
/// components.
///
/// # Fields
///
/// - `position`: Sign multipliers for XYZ position coordinates.
/// - `rotation`: Sign multipliers for quaternion components (excluding W).
///   Derived from axis flip combinations to maintain valid rotations.
/// - `spherical_harmonics`: Sign multipliers for the 15 spherical harmonic coefficients
///   (degrees 1-3). These follow specific symmetry rules based on axis flips.
///
/// # Example
///
/// ```
/// use spz::{CoordinateSystem, AxisFlips};
///
/// // Convert from PLY (RightDownFront) to GLB (LeftUpFront)
/// let flips = CoordinateSystem::RightDownFront.converter(CoordinateSystem::LeftUpFront);
///
/// // X and Y axes differ, Z matches
/// assert_eq!(flips.position, [-1.0, -1.0, 1.0]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct AxisFlips {
	/// Sign multipliers for XYZ position coordinates.
	pub position: [f32; 3],
	/// Sign multipliers for quaternion rotation components (X, Y, Z; W is unchanged).
	pub rotation: [f32; 3],
	/// Sign multipliers for spherical harmonic coefficients (15 values for degrees 1-3).
	pub spherical_harmonics: [f32; 15],
}

impl Default for AxisFlips {
	#[inline]
	fn default() -> Self {
		Self {
			position: [1.0, 1.0, 1.0],
			rotation: [1.0, 1.0, 1.0],
			spherical_harmonics: [1.0; 15],
		}
	}
}
