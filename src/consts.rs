// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Scale factor for DC color components.
///
/// To convert to RGB, we should multiply by 0.282, but it can
/// be useful to represent base colors that are out of range if the higher
/// spherical harmonics bands bring them back into range so we multiply by a
/// smaller value.
pub const COLOR_SCALE: f32 = 0.15;

pub const MAX_POINTS_TO_READ: i32 = 10_000_000;

/// "NGSP" in little-endian.
pub const PACKED_GAUSSIAN_HEADER_MAGIC: i32 = 0x5053474e;

pub const PACKED_GAUSSIAN_HEADER_VERSION: i32 = 3;

pub const EXTENSION: &str = "spz";

pub const MIME_TYPE: &str = "application/x-spz";
pub const MIME_MODEL: &str = "model/x-gaussian-splat";

pub mod flag {
	pub const ANTIALIASED: u8 = 0x1;
}
