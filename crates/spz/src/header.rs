// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::{Read, Write};

use anyhow::{Context, Result, bail};
use arbitrary::Arbitrary;
use serde::{Deserialize, Serialize};

use crate::consts;

pub const HEADER_SIZE: usize = std::mem::size_of::<PackedGaussiansHeader>();

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Arbitrary)]
#[repr(C)]
pub struct PackedGaussiansHeader {
	/// Always `0x5053474e`. "NGSP" = Niantic Gaussian SPlat.
	pub magic: i32,
	/// Currently, the only valid versions are 2 and 3.
	/// This crate only supports version 3.
	pub version: i32,
	/// The number of gaussians.
	pub num_points: i32,
	/// The degree of spherical harmonics.
	/// This must be between 0 and 3 (inclusive).
	pub spherical_harmonics_degree: u8,
	/// The number of bits used to store the fractional part of coordinates
	/// in the fixed-point encoding.
	pub fractional_bits: u8,
	/// A bit field containing flags.
	/// 	`0x1`: whether the splat was trained with antialiasing.
	pub flags: u8,
	/// Reserved for future use. Must be `0`.
	pub reserved: u8,
}

impl PackedGaussiansHeader {
	#[inline]
	pub fn read_from<R>(reader: &mut R) -> Result<Self>
	where
		R: Read,
	{
		let mut header_buf: [u8; HEADER_SIZE] = [0; HEADER_SIZE];

		match reader.read_exact(&mut header_buf) {
			Ok(_) => {},
			Err(err) => {
				bail!(err);
			},
		}
		Ok(unsafe { std::mem::transmute(header_buf) })
	}

	#[inline]
	pub fn serialize_to<W>(&self, stream: &mut W) -> Result<()>
	where
		W: Write,
	{
		let b = unsafe {
			std::slice::from_raw_parts(
				self as *const Self as *const u8,
				std::mem::size_of::<Self>(),
			)
		};
		stream.write_all(b)
			.with_context(|| "unable to write packed gaussians header to stream")
	}
}

impl Default for PackedGaussiansHeader {
	#[inline]
	fn default() -> Self {
		Self {
			magic: consts::HEADER_MAGIC,
			version: consts::SUPPORTED_SPZ_VERSION,
			num_points: 0,
			spherical_harmonics_degree: 0,
			fractional_bits: 0,
			flags: 0,
			reserved: 0,
		}
	}
}

impl From<PackedGaussiansHeader> for [u8; 16] {
	#[inline]
	fn from(from: PackedGaussiansHeader) -> Self {
		assert_eq!(
			std::mem::size_of::<Self>(),
			std::mem::size_of::<PackedGaussiansHeader>()
		);
		unsafe { std::mem::transmute::<PackedGaussiansHeader, Self>(from) }
	}
}

impl From<PackedGaussiansHeader> for &[u8] {
	#[inline]
	fn from(from: PackedGaussiansHeader) -> Self {
		assert_eq!(
			std::mem::size_of::<Self>(),
			std::mem::size_of::<PackedGaussiansHeader>()
		);
		unsafe { std::mem::transmute::<PackedGaussiansHeader, Self>(from) }
	}
}

impl From<[u8; 16]> for PackedGaussiansHeader {
	#[inline]
	fn from(from: [u8; 16]) -> Self {
		unsafe { std::mem::transmute::<[u8; 16], Self>(from) }
	}
}
