// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod compression;
pub mod consts;
pub mod coord;
pub mod gaussian_splat;
pub mod header;
pub mod math;
pub mod mmap;
pub mod packed;
pub mod unpacked;

pub use coord::{CoordinateConverter, CoordinateSystem};
pub use gaussian_splat::GaussianSplat;
pub use header::PackedGaussiansHeader;
pub use packed::{PackOptions, PackedGaussian, PackedGaussians};
pub use unpacked::{UnpackOptions, UnpackedGaussian};

pub mod prelude {
	pub use super::*;

	pub use super::coord::{CoordinateConverter, CoordinateSystem};
	pub use super::gaussian_splat::GaussianSplat;
	pub use super::header::PackedGaussiansHeader;
	pub use super::packed::{PackOptions, PackedGaussian, PackedGaussians};
	pub use super::unpacked::{UnpackOptions, UnpackedGaussian};
}
