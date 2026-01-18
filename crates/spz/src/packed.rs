// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Compressed (packed) Gaussian splat data for SPZ files.
//!
//! This module provides types for working with quantized, compressed splat data
//! as stored in SPZ files.
//!
//! The packed format uses:
//! - fixed-point encoding for positions
//! - quantized quaternions for rotations
//! - byte-quantized values for colors
//! - byte-quantized values for spherical harmonics,
//! achieving significant size reduction compared to raw floats.
//! ```

use std::io::BufReader;
use std::io::Read;
use std::io::Write;

use anyhow::bail;
use anyhow::{Context, Result};
use arbitrary::Arbitrary;
use likely_stable::{if_unlikely, unlikely};
use serde::{Deserialize, Serialize};

use crate::header::PackedGaussiansHeader;
use crate::{consts, math};
use crate::{coord::AxisFlips, unpacked::UnpackedGaussian};

static_assertions::const_assert_eq!(std::mem::size_of::<PackedGaussian>(), 65);

/// Intermediate representation. Represents a single low precision gaussian.
///
/// Coordinate system conversions are already applied at this stage.
/// Each gaussian has exactly 65 bytes, even if it does not have full spherical
/// harmonics.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PackedGaussian {
	pub position: [u8; 9],
	pub rotation: [u8; 4],
	pub scale: [u8; 3],
	pub color: [u8; 3],
	pub alpha: u8,
	pub sh_r: [u8; 15],
	pub sh_g: [u8; 15],
	pub sh_b: [u8; 15],
}

impl PackedGaussian {
	/// Decompresses this packed Gaussian into full-precision values,
	/// [`UnpackedGaussian`].
	///
	/// Decodes quantized position, rotation, scale, color, alpha, and spherical
	/// harmonics back to `f32` values, applying coordinate axis flips as specified.
	///
	/// # Args
	///
	/// * `uses_float16` — If `true`, positions are float16 (6 bytes); otherwise fixed24 (9 bytes)
	/// * `uses_quaternion_smallest_three` — If `true`, rotations use 4-byte smallest-three encoding
	/// * `fractional_bits` — Bits for fractional part in fixed-point position encoding
	/// * `coord_flip` — Sign multipliers for coordinate system transformation
	///
	/// # Returns
	///
	/// An [`UnpackedGaussian`] with all attributes decoded to `f32`.
	pub fn unpack(
		&self,
		uses_float16: bool,
		uses_quaternion_smallest_three: bool,
		fractional_bits: i32,
		coord_flip: &AxisFlips,
	) -> Result<UnpackedGaussian> {
		let mut result = UnpackedGaussian::default();

		// positions
		if uses_float16 {
			for i in 0..3 {
				let lo = self.position[i * 2] as u16;
				let hi = self.position[i * 2 + 1] as u16;
				let half = lo | (hi << 8);

				result.position[i] =
					coord_flip.position[i] * math::half_to_float(half);
			}
		} else {
			let s = 1u32 << (fractional_bits as u32);

			if s == 0 {
				bail!("invalid fractional bits (= 0): {}", fractional_bits);
			}
			let scale = 1.0_f32 / s as f32;

			for i in 0..3 {
				let b0 = self.position[i * 3 + 0] as i32;
				let b1 = (self.position[i * 3 + 1] as i32) << 8;
				let b2 = (self.position[i * 3 + 2] as i32) << 16;

				let mut fixed32 = b0 | b1 | b2;

				if (fixed32 & 0x800000) != 0 {
					fixed32 |= 0xff000000u32 as i32;
				}
				result.position[i] =
					coord_flip.position[i] * (fixed32 as f32 * scale);
			}
		}
		// scales
		for i in 0..3 {
			result.scale[i] = (self.scale[i] as f32 / 16.0_f32) - 10.0_f32;
		}
		// rotation
		math::unpack_quaternion_smallest_three_with_flip(
			&mut result.rotation,
			if uses_quaternion_smallest_three {
				&self.rotation
			} else {
				&self.rotation[..3]
			},
			coord_flip.rotation,
		);
		// alpha
		result.alpha = math::inv_sigmoid(self.alpha as f32 / 255.0_f32);

		// color
		for i in 0..3 {
			result.color[i] = ((self.color[i] as f32 / 255.0_f32) - 0.5_f32)
				/ consts::COLOR_SCALE;
		}
		// spherical harmonics
		for i in 0..15 {
			let f = coord_flip.spherical_harmonics[i];

			result.sh_r[i] = f * math::unquantize_sh(self.sh_r[i]);
			result.sh_g[i] = f * math::unquantize_sh(self.sh_g[i]);
			result.sh_b[i] = f * math::unquantize_sh(self.sh_b[i]);
		}
		Ok(result)
	}
}

/// Compressed Gaussian splat collection in SPZ format.
///
/// Stores all splat data in non-interleaved arrays for efficient compression.
/// Each attribute (positions, rotations, etc.) is stored contiguously across
/// all splats rather than per-splat.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct PackedGaussians {
	/// Total number of Gaussian splats.
	pub num_points: i32,
	/// Spherical harmonics degree (0-3).
	pub sh_degree: i32,
	/// Bits used for fractional part of fixed-point positions.
	pub fractional_bits: i32,
	/// Whether splats use mip-splatting antialiasing.
	pub antialiased: bool,
	/// Whether rotations use smallest-three quaternion encoding.
	pub uses_quaternion_smallest_three: bool,

	/// Quantized positions (6 or 9 bytes per splat).
	pub positions: Vec<u8>,
	/// Quantized log-scales (3 bytes per splat).
	pub scales: Vec<u8>,
	/// Packed quaternion rotations (3 or 4 bytes per splat).
	pub rotations: Vec<u8>,
	/// Quantized opacity (1 byte per splat).
	pub alphas: Vec<u8>,
	/// Quantized RGB colors (3 bytes per splat).
	pub colors: Vec<u8>,
	/// Quantized spherical harmonics (variable size based on degree).
	pub spherical_harmonics: Vec<u8>,
}

impl PackedGaussians {
	/// Deserializes packed Gaussian data from gzip-compressed bytes.
	///
	/// `bytes` - gzip compressed, packed gaussian data.
	pub fn from_bytes<B>(bytes: B) -> Result<Self>
	where
		B: AsRef<[u8]>,
	{
		if unlikely(bytes.as_ref().is_empty()) {
			// we cannot return an empty struct as there is no header
			bail!("data is empty");
		}
		let mut decompressed = Vec::<u8>::new();

		crate::compression::gzip_to_bytes(bytes, &mut decompressed)
			.with_context(|| "unable to decompress gzip data")?;

		let packed: Self = decompressed
			.try_into()
			.with_context(|| "unable to parse packed gaussian data")?;

		Ok(packed)
	}

	/// Constructs an SPZ header from this packed data's metadata.
	#[inline]
	pub fn construct_header(&self) -> PackedGaussiansHeader {
		PackedGaussiansHeader {
			num_points: self.num_points,
			spherical_harmonics_degree: self.sh_degree as u8,
			fractional_bits: self.fractional_bits as u8,
			flags: if self.antialiased {
				consts::flag::ANTIALIASED
			} else {
				0
			},
			reserved: 0,
			..Default::default()
		}
	}

	/// Serializes to a complete SPZ file as a byte vector.
	///
	/// Returns header followed by all attribute arrays in SPZ order.
	pub fn as_bytes_vec(&self) -> Result<Vec<u8>> {
		let mut ret = Vec::new();

		self.construct_header().serialize_to(&mut ret)?;

		ret.extend_from_slice(&self.positions);
		ret.extend_from_slice(&self.alphas);
		ret.extend_from_slice(&self.colors);
		ret.extend_from_slice(&self.scales);
		ret.extend_from_slice(&self.rotations);
		ret.extend_from_slice(&self.spherical_harmonics);

		Ok(ret)
	}

	/// Writes this packed data to a writer in SPZ format.
	pub fn write_self_to<W>(&self, stream: &mut W) -> Result<()>
	where
		W: Write,
	{
		self.construct_header().serialize_to(stream)?;

		stream.write_all(&self.positions)?;
		stream.write_all(&self.alphas)?;
		stream.write_all(&self.colors)?;
		stream.write_all(&self.scales)?;
		stream.write_all(&self.rotations)?;
		stream.write_all(&self.spherical_harmonics)?;

		Ok(())
	}

	/// Returns `true` if positions are stored as float16.
	#[inline]
	pub fn uses_float16(&self) -> bool {
		self.positions.len() == self.num_points as usize * 3 * 2
	}

	/// Returns the packed data for a single splat at index `i`.
	pub fn at(&self, i: usize) -> Result<PackedGaussian> {
		if i >= self.num_points as usize {
			bail!("index out of bounds: {}", i);
		}
		let idx = i as usize;
		let mut result = PackedGaussian::default();
		let position_bytes = if self.uses_float16() { 6 } else { 9 };
		let p_start = idx.saturating_mul(position_bytes);

		if p_start != usize::MAX
			&& let Some(slice) = self.positions.get(p_start..p_start + position_bytes)
		{
			result.position[..position_bytes].copy_from_slice(slice);
		}
		let start3 = idx.saturating_mul(3);

		if let Some(slice) = self.scales.get(start3..start3 + 3) {
			result.scale.copy_from_slice(slice);
		}
		let rotation_bytes = if self.uses_quaternion_smallest_three {
			4
		} else {
			3
		};
		let r_start = idx.saturating_mul(rotation_bytes);

		if r_start != usize::MAX
			&& let Some(slice) = self.rotations.get(r_start..r_start + rotation_bytes)
		{
			result.rotation[..rotation_bytes].copy_from_slice(slice);
		}
		if let Some(slice) = self.colors.get(start3..start3 + 3) {
			result.color.copy_from_slice(slice);
		}
		if let Some(a) = self.alphas.get(idx) {
			result.alpha = *a;
		}
		let sh_dim = math::dim_for_degree(self.sh_degree as u8) as usize;
		let base_point_sh = idx.saturating_mul(sh_dim).saturating_mul(3);

		for j in 0..sh_dim {
			let base = base_point_sh.saturating_add(j.saturating_mul(3));

			if let Some(slice) = self.spherical_harmonics.get(base..base + 3) {
				result.sh_r[j] = slice[0];
				result.sh_g[j] = slice[1];
				result.sh_b[j] = slice[2];
			} else {
				result.sh_r[j] = 128;
				result.sh_g[j] = 128;
				result.sh_b[j] = 128;
			}
		}
		for j in sh_dim..15 {
			result.sh_r[j] = 128;
			result.sh_g[j] = 128;
			result.sh_b[j] = 128;
		}
		Ok(result)
	}

	/// Unpacks a single splat at index `i` with coordinate transformation.
	///
	/// Applies the given axis flips during decompression.
	#[inline]
	pub fn unpack(&self, i: usize, coord_flip: &AxisFlips) -> Result<UnpackedGaussian> {
		self.at(i)?.unpack(
			self.uses_float16(),
			self.uses_quaternion_smallest_three,
			self.fractional_bits,
			coord_flip,
		)
	}

	/// Validates that all internal arrays have the expected sizes.
	///
	/// Returns `true` if sizes match the expected layout for the given
	/// number of points and SH dimension.
	pub fn check_sizes(&self, num_points: usize, sh_dim: u8, uses_float16: bool) -> bool {
		let pos_expected = num_points * 3 * if uses_float16 { 2 } else { 3 };
		let scales_expected = num_points * 3;
		let rot_expected = num_points
			* if self.uses_quaternion_smallest_three {
				4
			} else {
				3
			};
		let alphas_expected = num_points;
		let colors_expected = num_points * 3;
		let sh_expected = num_points * (sh_dim as usize) * 3;

		if self.positions.len() != pos_expected
			|| self.scales.len() != scales_expected
			|| self.rotations.len() != rot_expected
			|| self.alphas.len() != alphas_expected
			|| self.colors.len() != colors_expected
			|| self.spherical_harmonics.len() != sh_expected
		{
			return false;
		}
		true
	}
}

impl TryFrom<Vec<u8>> for PackedGaussians {
	type Error = anyhow::Error;

	fn try_from(b: Vec<u8>) -> Result<Self, Self::Error> {
		Self::try_from(b.as_slice())
	}
}

impl TryFrom<&[u8]> for PackedGaussians {
	type Error = anyhow::Error;

	fn try_from(b: &[u8]) -> Result<Self, Self::Error> {
		let mut from_reader = BufReader::new(b);

		let header = PackedGaussiansHeader::read_from(&mut from_reader)
			.with_context(|| "unable to read packed gaussians header")?;

		if unlikely(header.magic != consts::HEADER_MAGIC) {
			bail!("invalid magic number in packed gaussians header");
		}
		if unlikely(header.version < 1 || header.version > 3) {
			bail!("unsupported version: {}", header.version);
		}
		if unlikely(header.spherical_harmonics_degree > 3) {
			bail!(
				"unsupported spherical harmonics degree: {}",
				header.spherical_harmonics_degree
			);
		}
		let num_points = header.num_points;
		let uses_float16 = header.version == 1;
		let uses_quaternion_smallest_three = header.version >= 3;

		let mut result = PackedGaussians {
			num_points,
			sh_degree: header.spherical_harmonics_degree as i32,
			fractional_bits: header.fractional_bits as i32,
			antialiased: (header.flags & consts::flag::ANTIALIASED) != 0,
			uses_quaternion_smallest_three: uses_quaternion_smallest_three,
			positions: vec![
				0;
				num_points as usize * 3 * if uses_float16 { 2 } else { 3 }
			],
			scales: vec![0; (num_points as usize) * 3],
			rotations: vec![
				0;
				num_points as usize
					* if uses_quaternion_smallest_three { 4 } else { 3 }
			],
			alphas: vec![0; num_points as usize],
			colors: vec![0; (num_points as usize) * 3],
			spherical_harmonics: vec![
				0;
				num_points as usize
					* math::dim_for_degree(header.spherical_harmonics_degree)
						as usize * 3
			],
		};
		if_unlikely! { let Err(err) = from_reader.read_exact(&mut result.positions) => {
			bail!("read error (positions): {}", err);
		}};
		if_unlikely! { let Err(err) = from_reader.read_exact(&mut result.alphas) => {
			bail!("read error (alphas): {}", err);
		}};
		if_unlikely! { let Err(err) = from_reader.read_exact(&mut result.colors) => {
			bail!("read error (colors): {}", err);
		}};
		if_unlikely! { let Err(err) = from_reader.read_exact(&mut result.scales) => {
			bail!("read error (scales): {}", err);
		}};
		if_unlikely! { let Err(err) = from_reader.read_exact(&mut result.rotations) => {
			bail!("read error (rotations): {}", err);
		}};
		if_unlikely! { let Err(err) = from_reader.read_exact(&mut result.spherical_harmonics) => {
			bail!("read error (sh): {}", err);
		}};
		Ok(result)
	}
}
