// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Scale factor for DC color components.
///
/// To convert to RGB, we should multiply by 0.282, but it can
/// be useful to represent base colors that are out of range if the higher
/// spherical harmonics bands bring them back into range so we multiply by a
/// smaller value.
pub const COLOR_SCALE: f32 = 0.15;

/// "NGSP" in little-endian.
pub const HEADER_MAGIC: i32 = 0x5053474e;

/// Supported .spz version. This crate currently only supports version 3.
pub const SUPPORTED_SPZ_VERSION: i32 = 3;

pub const EXTENSION: &str = "spz";

pub mod flag {
	pub const ANTIALIASED: u8 = 0x1;
}
