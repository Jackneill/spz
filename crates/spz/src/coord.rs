// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::str::FromStr;

use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// Supported 3D coordinate systems for Gaussian splat data.
///
/// To aid with coordinate system conversions, callers should specify the
/// coordinate system their Gaussian Splat data is represented in when saving
/// and what coordinate system their rendering system uses when loading.
///
/// These are specified in the [`LoadOptions`](crate::gaussian_splat::LoadOptions)
/// and [`SaveOptions`](crate::gaussian_splat::SaveOptions) respectively.
/// If the coordinate system is `Unspecified`, data will be saved and loaded
/// without conversion, which may harm interoperability.
///
/// Enum item values follow the original Niantic C++ SPZ values.
#[derive(
	EnumIter, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Arbitrary,
)]
pub enum CoordinateSystem {
	#[default]
	Unspecified = 0,

	/// LDB.
	LeftDownBack = 1,
	/// RDB.
	RightDownBack = 2,
	/// LUB.
	LeftUpBack = 3,
	/// RUB. SPZ Internal, OpenGL, Three.js.
	RightUpBack = 4,
	/// LDF.
	LeftDownFront = 5,
	/// RDF. PLY coordinate system.
	RightDownFront = 6,
	/// LUF. GLB coordinate system.
	LeftUpFront = 7,
	/// RUF. Unity coordinate system.
	RightUpFront = 8,
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
	pub const fn as_short_str(&self) -> &'static str {
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
	pub const fn axis_flips_to(self, target: CoordinateSystem) -> AxisFlips {
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
	pub const fn axes_align(self, other: CoordinateSystem) -> (bool, bool, bool) {
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

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::rstest;
	use strum::IntoEnumIterator;

	#[rstest]
	#[case(CoordinateSystem::Unspecified, "UNSPECIFIED")]
	#[case(CoordinateSystem::LeftDownBack, "LDB")]
	#[case(CoordinateSystem::RightDownBack, "RDB")]
	#[case(CoordinateSystem::LeftUpBack, "LUB")]
	#[case(CoordinateSystem::RightUpBack, "RUB")]
	#[case(CoordinateSystem::LeftDownFront, "LDF")]
	#[case(CoordinateSystem::RightDownFront, "RDF")]
	#[case(CoordinateSystem::LeftUpFront, "LUF")]
	#[case(CoordinateSystem::RightUpFront, "RUF")]
	fn test_as_short_str(#[case] cs: CoordinateSystem, #[case] expected: &str) {
		assert_eq!(cs.as_short_str(), expected);
	}

	#[rstest]
	#[case("RUB", CoordinateSystem::RightUpBack)]
	#[case("rub", CoordinateSystem::RightUpBack)]
	#[case("RightUpBack", CoordinateSystem::RightUpBack)]
	#[case("RIGHT-UP-BACK", CoordinateSystem::RightUpBack)]
	#[case("RIGHT_UP_BACK", CoordinateSystem::RightUpBack)]
	#[case("LDF", CoordinateSystem::LeftDownFront)]
	#[case("RDF", CoordinateSystem::RightDownFront)]
	#[case("LUF", CoordinateSystem::LeftUpFront)]
	#[case("RUF", CoordinateSystem::RightUpFront)]
	#[case("LDB", CoordinateSystem::LeftDownBack)]
	#[case("RDB", CoordinateSystem::RightDownBack)]
	#[case("LUB", CoordinateSystem::LeftUpBack)]
	#[case("nonsense", CoordinateSystem::Unspecified)]
	#[case("", CoordinateSystem::Unspecified)]
	fn test_from_str(#[case] input: &str, #[case] expected: CoordinateSystem) {
		assert_eq!(input.parse::<CoordinateSystem>().unwrap(), expected);
	}

	#[test]
	fn test_from_str_ref() {
		let cs: CoordinateSystem = "RDF".into();

		assert_eq!(cs, CoordinateSystem::RightDownFront);
	}

	#[test]
	fn test_display_all_variants() {
		for cs in CoordinateSystem::iter() {
			let display = format!("{}", cs);

			assert!(
				!display.is_empty(),
				"Display should not be empty for {:?}",
				cs
			);
		}
	}

	#[rstest]
	#[case(CoordinateSystem::RightDownFront, "Right-Down-Front")]
	#[case(CoordinateSystem::LeftUpFront, "Left-Up-Front")]
	#[case(CoordinateSystem::Unspecified, "Unspecified")]
	fn test_display_specific(#[case] cs: CoordinateSystem, #[case] expected: &str) {
		assert_eq!(format!("{}", cs), expected);
	}

	#[test]
	fn test_axes_align_same_system() {
		for cs in CoordinateSystem::iter() {
			if cs == CoordinateSystem::Unspecified {
				continue;
			}
			let (x, y, z) = cs.axes_align(cs);

			assert!(
				x && y && z,
				"same system should have all axes aligned for {:?}",
				cs
			);
		}
	}

	#[test]
	fn test_axes_align_unspecified_always_true() {
		for cs in CoordinateSystem::iter() {
			assert_eq!(
				CoordinateSystem::Unspecified.axes_align(cs),
				(true, true, true),
				"Unspecified should always match for {:?}",
				cs
			);
			assert_eq!(
				cs.axes_align(CoordinateSystem::Unspecified),
				(true, true, true),
				"Anything vs Unspecified should always match for {:?}",
				cs
			);
		}
	}

	#[rstest]
	#[case(
		CoordinateSystem::RightUpBack,
		CoordinateSystem::LeftUpFront,
		(false, true, false)
	)]
	#[case(
		CoordinateSystem::RightUpBack,
		CoordinateSystem::RightDownFront,
		(true, false, false)
	)]
	#[case(
		CoordinateSystem::LeftDownBack,
		CoordinateSystem::RightUpFront,
		(false, false, false)
	)]
	#[case(
		CoordinateSystem::LeftDownBack,
		CoordinateSystem::LeftDownBack,
		(true, true, true)
	)]
	fn test_axes_align_known(
		#[case] a: CoordinateSystem,
		#[case] b: CoordinateSystem,
		#[case] expected: (bool, bool, bool),
	) {
		assert_eq!(a.axes_align(b), expected);
	}

	#[test]
	fn test_axes_align_symmetric() {
		for a in CoordinateSystem::iter() {
			for b in CoordinateSystem::iter() {
				assert_eq!(
					a.axes_align(b),
					b.axes_align(a),
					"axes_align should be symmetric for {:?} vs {:?}",
					a,
					b
				);
			}
		}
	}

	#[test]
	fn test_axis_flips_same_system_is_identity() {
		for cs in CoordinateSystem::iter() {
			if cs == CoordinateSystem::Unspecified {
				continue;
			}
			let flips = cs.axis_flips_to(cs);

			assert_eq!(
				flips.position,
				[1.0, 1.0, 1.0],
				"position flips should be identity for {:?}",
				cs
			);
			assert_eq!(
				flips.rotation,
				[1.0, 1.0, 1.0],
				"rotation flips should be identity for {:?}",
				cs
			);
		}
	}

	#[test]
	fn test_axis_flips_position_signs() {
		let flips = CoordinateSystem::RightUpBack
			.axis_flips_to(CoordinateSystem::RightDownFront);

		assert_eq!(flips.position[0], 1.0); // R→R
		assert_eq!(flips.position[1], -1.0); // U→D
		assert_eq!(flips.position[2], -1.0); // B→F
	}

	#[test]
	fn test_axis_flips_rotation_derived_from_position() {
		let flips = CoordinateSystem::RightUpBack
			.axis_flips_to(CoordinateSystem::LeftDownFront);

		assert_eq!(flips.rotation[0], flips.position[1] * flips.position[2]);
		assert_eq!(flips.rotation[1], flips.position[0] * flips.position[2]);
		assert_eq!(flips.rotation[2], flips.position[0] * flips.position[1]);
	}

	#[test]
	fn test_axis_flips_default_is_identity() {
		let flips = AxisFlips::default();

		assert_eq!(flips.position, [1.0, 1.0, 1.0]);
		assert_eq!(flips.rotation, [1.0, 1.0, 1.0]);
		assert_eq!(flips.spherical_harmonics, [1.0; 15]);
	}
}
