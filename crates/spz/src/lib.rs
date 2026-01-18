// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SPZ file format crate for Gaussian Splat data.

pub mod compression;
pub mod consts;
pub mod coord;
pub mod gaussian_splat;
pub mod header;
pub mod math;
pub mod mmap;
pub mod packed;
pub mod unpacked;

pub mod prelude {
	pub use super::*;

	pub use super::coord::{AxisFlips, CoordinateSystem};
	pub use super::gaussian_splat::{BoundingBox, GaussianSplat, LoadOptions, SaveOptions};
	pub use super::header::PackedGaussiansHeader;
	pub use super::packed::{PackedGaussian, PackedGaussians};
	pub use super::unpacked::UnpackedGaussian;
}
