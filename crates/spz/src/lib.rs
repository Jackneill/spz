// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SPZ file format library for Gaussian splat data.
//!
//! # Quick Start
//!
//! ```rust
//! use spz::prelude::*;
//! ```
//!
//! # Modules
//!
//! - [`coord`] — Coordinate system conversions ([`CoordinateSystem`], [`AxisFlips`])
//! - [`gaussian_splat`] — Core splat type ([`GaussianSplat`], [`BoundingBox`])
//! - [`packed`] — Compressed SPZ types ([`PackedGaussians`], [`PackOptions`])
//! - [`unpacked`] — Decompressed SPZ types ([`UnpackedGaussian`], [`UnpackOptions`])
//! - [`header`] — SPZ file header ([`PackedGaussiansHeader`])
//! - [`compression`] — Internal compression utilities
//! - [`math`] — Internal math utils
//! - [`mmap`] — Memory-mapped file reading
//!
//! [`CoordinateSystem`]: coord::CoordinateSystem
//! [`AxisFlips`]: coord::AxisFlips
//! [`GaussianSplat`]: gaussian_splat::GaussianSplat
//! [`BoundingBox`]: gaussian_splat::BoundingBox
//! [`PackedGaussians`]: packed::PackedGaussians
//! [`PackOptions`]: packed::PackOptions
//! [`UnpackedGaussian`]: unpacked::UnpackedGaussian
//! [`UnpackOptions`]: unpacked::UnpackOptions
//! [`PackedGaussiansHeader`]: header::PackedGaussiansHeader

pub mod compression;
pub mod consts;
pub mod coord;
pub mod gaussian_splat;
pub mod header;
pub mod math;
pub mod mmap;
pub mod packed;
pub mod unpacked;

/// Common imports for working with SPZ files.
///
/// ```rust
/// use spz::prelude::*;
/// ```
pub mod prelude {
	pub use super::*;

	pub use super::coord::{AxisFlips, CoordinateSystem};
	pub use super::gaussian_splat::{BoundingBox, GaussianSplat};
	pub use super::header::PackedGaussiansHeader;
	pub use super::packed::{PackOptions, PackedGaussian, PackedGaussians};
	pub use super::unpacked::{UnpackOptions, UnpackedGaussian};
}
