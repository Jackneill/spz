// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SPZ file header parsing and serialization.
//!
//! This module provides the [`PackedGaussiansHeader`] struct for reading and
//! writing the 16-byte header that prefixes every SPZ file. The header contains
//! essential metadata for decoding the compressed Gaussian splat data.
//!
//! # File Format
//!
//! SPZ files begin with a fixed 16-byte header followed by compressed splat data.
//! The header uses little-endian (LE) byte order that can be directly memory-mapped
//! due to its `#[repr(C)]` layout.
//!
//! # Example
//!
//! ```no_run
//! use std::fs::File;
//! use spz::header::{PackedGaussiansHeader, HEADER_SIZE};
//!
//! // Read header from file
//! let mut file = File::open("scene.spz").unwrap();
//! let header = PackedGaussiansHeader::read_from(&mut file).unwrap();
//!
//! // Create and write a new header
//! let header = PackedGaussiansHeader::default();
//! let mut output = File::create("output.spz").unwrap();
//!
//! header.serialize_to(&mut output).unwrap();
//! ```

use std::{
	fmt::Display,
	io::{Read, Write},
	path::Path,
};

use anyhow::{Context, Result, bail};
use arbitrary::Arbitrary;
use bitflags::bitflags;
use likely_stable::{likely, unlikely};
use serde::{Deserialize, Serialize};

use crate::mmap::mmap_range;

/// Header Magic Value. "NGSP" in little-endian (LE).
/// Every SPZ file's 1st 4 bytes are this magic number.
pub const MAGIC_VALUE: i32 = 0x5053474e;

/// A bit field containing flags for SPZ files.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Arbitrary)]
pub struct Flags(pub u8);

bitflags! {
	impl Flags: u8 {
		/// Whether the splat was trained with antialiasing.
		const ANTIALIASED = 0x1;
	}
}

impl Flags {
	/// Returns a new Flags with no flags set.
	#[inline]
	pub fn none() -> Self {
		Self(0)
	}

	/// Checks if the antialiased flag is set.
	#[inline]
	pub fn is_antialiased(&self) -> bool {
		self.contains(Flags::ANTIALIASED)
	}
}

/// SPZ versions.
/// Currently, the only valid versions are v2 and v3.
/// This crate currently only supports version `v3`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Arbitrary)]
#[repr(i32)]
pub enum Version {
	/// Version 1 of the SPZ file format. Unsupported.
	V1 = 1,

	/// Version 2 of the SPZ file format. Unsupported.
	V2 = 2,

	/// Version 3 of the SPZ file format. **Supported** by this crate.
	V3 = 3,
}

impl Display for Version {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Version::V1 => write!(f, "1"),
			Version::V2 => write!(f, "2"),
			Version::V3 => write!(f, "3"),
		}
	}
}

static_assertions::const_assert_eq!(HEADER_SIZE, 16);

/// Size of the SPZ header in bytes (16).
pub const HEADER_SIZE: usize = std::mem::size_of::<Header>();

/// Fixed-size 16-byte header for SPZ (Gaussian Splat) files.
///
/// This header appears at the start of every SPZ file and contains metadata
/// needed to decode the compressed Gaussian Splat data that follows. The
/// struct uses repr C for direct memory-mapped reading/writing.
///
/// See also [`GaussianSplat`](crate::gaussian_splat::GaussianSplat) for the
/// full splat data structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Arbitrary)]
#[repr(C)]
pub struct Header {
	/// Always `0x5053474e`. "NGSP" = Niantic Gaussian SPlat.
	pub magic: i32,
	/// Currently, the only valid versions are v2 and v3.
	/// This crate only supports version `v3`.
	pub version: Version,
	/// The number of gaussians.
	pub num_points: i32,
	/// The degree of spherical harmonics.
	/// This must be between 0 and 3 (inclusive).
	pub spherical_harmonics_degree: u8,
	/// The number of bits used to store the fractional part of coordinates
	/// in the fixed-point encoding.
	pub fractional_bits: u8,
	/// A bit field containing flags.
	/// - `0x1`: whether the splat was trained with antialiasing.
	pub flags: Flags,
	/// Reserved for future use. Must be `0`.
	pub reserved: u8,
}

impl Header {
	/// Reads a header directly from a file path using memory mapping.
	///
	/// Memory-maps the file and reads the 1st 16 bytes as a header.
	/// Efficient for quickly inspecting SPZ file metadata without
	/// reading the entire file.
	///
	/// DOES NOT validate whether the read header is a valid SPZ header,
	/// simply reads the bytes and interprets them as a header.
	#[inline]
	pub fn from_file_unchecked<P>(filepath: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		let mmap = mmap_range(&filepath, 0, HEADER_SIZE)
			.with_context(|| "unable to memory-map file header range")?;

		if unlikely(mmap.len() != HEADER_SIZE) {
			bail!(
				"files is of invalid length, expected {} bytes, got {}",
				HEADER_SIZE,
				mmap.len()
			);
		}
		Ok(mmap.as_ref().into())
	}

	/// Reads a header directly from a file path using memory mapping.
	///
	/// Memory-maps the file and reads the 1st 16 bytes as a header.
	/// Efficient for quickly inspecting SPZ file metadata without
	/// reading the entire file.
	#[inline]
	pub fn from_file<P>(filepath: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		let ret = Self::from_file_unchecked(filepath)?;

		if unlikely(!ret.is_valid()) {
			bail!("header fails validation");
		}
		Ok(ret)
	}

	/// Does some basic validation of this header.
	#[inline]
	pub fn is_valid(&self) -> bool {
		likely(self.magic == MAGIC_VALUE
			&& matches!(self.version, Version::V2 | Version::V3)
			&& (0..=3).contains(&self.spherical_harmonics_degree)
			&& self.reserved == 0)
	}

	/// Reads a header from the given reader.
	///
	/// Consumes exactly [`HEADER_SIZE`] (16 bytes) from the reader and
	/// interprets them as a header. Does not validate the magic number
	/// or version, for that use a separate validation if needed.
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
		Ok(unsafe { std::mem::transmute::<[u8; HEADER_SIZE], Self>(header_buf) })
	}

	/// Writes this header to the given writer.
	///
	/// Writes exactly [`HEADER_SIZE`] (16 bytes) in the binary format
	/// expected by SPZ readers.
	#[inline]
	pub fn serialize_to<W>(&self, stream: &mut W) -> Result<()>
	where
		W: Write,
	{
		let b = unsafe { std::mem::transmute::<Self, [u8; HEADER_SIZE]>(self.clone()) };

		stream.write_all(b.as_slice())
			.with_context(|| "unable to write header to stream")?;

		stream.flush()
			.with_context(|| "unable to flush header to stream")
	}
}

impl Default for Header {
	#[inline]
	fn default() -> Self {
		Self {
			magic: MAGIC_VALUE,
			version: Version::V3,
			num_points: 0,
			spherical_harmonics_degree: 0,
			fractional_bits: 0,
			flags: Flags(0),
			reserved: 0,
		}
	}
}

impl From<Header> for [u8; 16] {
	#[inline]
	fn from(from: Header) -> Self {
		unsafe { std::mem::transmute::<Header, Self>(from) }
	}
}

impl From<Header> for &[u8] {
	#[inline]
	fn from(from: Header) -> Self {
		unsafe { std::mem::transmute::<Header, Self>(from) }
	}
}

impl From<[u8; 16]> for Header {
	#[inline]
	fn from(from: [u8; 16]) -> Self {
		unsafe { std::mem::transmute::<[u8; 16], Self>(from) }
	}
}

impl From<&[u8]> for Header {
	#[inline]
	fn from(from: &[u8]) -> Self {
		debug_assert_eq!(from.len(), HEADER_SIZE);

		unsafe { std::mem::transmute::<&[u8], Self>(from) }
	}
}
