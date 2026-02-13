// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cap'n Proto integration for the SPZ Gaussian Splat file format.
//!
//! This crate provides generated Cap'n Proto types for the SPZ schema and,
//! when the `spz` feature is enabled, convenience conversion functions between
//! the native `spz` crate types and their Cap'n Proto counterparts.

#[cfg(feature = "spz")]
pub use spz;

pub mod generated {
	pub use super::spz_capnp::*;
}

#[cfg(feature = "spz")]
pub mod convert;

::capnp::generated_code!(pub mod spz_capnp);
