// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::Write;
use std::{fmt::Display, io::Read, path::Path};

use anyhow::{Context, Error, Result, bail};
use arbitrary::Arbitrary;
use bitflags::bitflags;
use likely_stable::{likely, unlikely};
use serde::{Deserialize, Serialize};
use zerocopy::{FromBytes, IntoBytes, KnownLayout, TryFromBytes};

use crate::compression;
use crate::mmap::mmap_range;

/// Header Magic Value. "NGSP" in little-endian (LE).
/// Every SPZ file's 1st 4 bytes are this magic number.
pub const MAGIC_VALUE: i32 = 0x5053474e;

/// A bit field containing flags for SPZ files.
#[derive(
	Debug,
	Default,
	Clone,
	Copy,
	PartialEq,
	Eq,
	Hash,
	Serialize,
	Deserialize,
	Arbitrary,
	FromBytes,
	IntoBytes,
	KnownLayout,
)]
pub struct Flags(pub u8);

bitflags! {
	impl Flags: u8 {
		/// Whether the Gaussian Splat was trained with `antialiasing`.
		const ANTIALIASED = 0x1;
	}
}

impl Flags {
	/// Returns a new Flags with no flags set.
	#[inline]
	pub const fn none() -> Self {
		Self(0)
	}

	/// Checks if the antialiased flag is set.
	#[inline]
	pub const fn is_antialiased(&self) -> bool {
		self.contains(Flags::ANTIALIASED)
	}

	/// Validates that only known/defined flags are set.
	#[inline]
	pub fn is_valid(&self) -> bool {
		// currently, only bit 0 is defined as is_antialiased
		(self.0 & !Flags::ANTIALIASED.bits()) == 0
	}
}

/// SPZ versions.
/// Currently, the only valid versions are v2 and v3.
/// This crate currently only supports version `v3`.
#[derive(
	Debug,
	Clone,
	Default,
	Copy,
	PartialEq,
	Eq,
	Hash,
	Serialize,
	Deserialize,
	Arbitrary,
	TryFromBytes,
	IntoBytes,
	KnownLayout,
)]
#[repr(i32)]
pub enum Version {
	/// Version 1 of the SPZ file format. Unsupported.
	V1 = 1,

	/// Version 2 of the SPZ file format. **Supported** by this crate.
	V2 = 2,

	/// Version 3 of the SPZ file format. **Supported** by this crate.
	#[default]
	V3 = 3,
}

impl Display for Version {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Version::V1 => write!(f, "v1"),
			Version::V2 => write!(f, "v2"),
			Version::V3 => write!(f, "v3"),
		}
	}
}

/// Enough bytes to decompress the first 16 bytes of the spz file, the header.
const COMPRESSED_BLOCK_READ_SIZE: u16 = 512;

// Asserts that the size of the `Header` struct is 16 bytes (by specification)
// at compile time.
static_assertions::const_assert_eq!(16, HEADER_SIZE);

/// Size of the SPZ header in bytes (16).
pub const HEADER_SIZE: usize = std::mem::size_of::<Header>();

/// Fixed-size 16-byte header for SPZ (Gaussian Splat) files.
///
/// This header appears at the start of every SPZ file (uncompressed) and
/// contains metadata needed to decode the Gaussian Splat data that follows.
/// `repr C` is used for direct memory-mapped reading/writing.
///
/// See also [`GaussianSplat`](crate::gaussian_splat::GaussianSplat) for the
/// full Gaussian Splat data structure.
#[derive(
	Clone,
	Copy,
	Debug,
	PartialEq,
	Eq,
	Serialize,
	Deserialize,
	Arbitrary,
	TryFromBytes,
	IntoBytes,
	KnownLayout,
)]
#[repr(C)]
pub struct Header {
	/// Always `0x5053474e`. "NGSP" = Niantic Gaussian SPlat.
	pub magic: i32,
	/// Currently, the only valid versions are v2 and v3.
	/// This crate supports version `v2` and `v3`.
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
	/// Decompresses and reads a header from the given compressed bytes.
	///
	/// Does NOT validate whether the read header is a valid SPZ header,
	/// simply reads the bytes and interprets them as a header.
	#[inline]
	pub fn from_compressed_bytes_unchecked<C>(compressed: C) -> Result<Self>
	where
		C: AsRef<[u8]>,
	{
		let mut decompressed = [0_u8; COMPRESSED_BLOCK_READ_SIZE as usize];

		compression::gzip::decompress(compressed, &mut decompressed)
			.with_context(|| "unable to decompress header bytes")?;

		decompressed[..HEADER_SIZE]
			.try_into()
			.with_context(|| "unable to read header")
	}

	/// Decompresses and reads a header from the given compressed bytes.
	#[inline]
	pub fn from_compressed_bytes<C>(compressed: C) -> Result<Self>
	where
		C: AsRef<[u8]>,
	{
		let header = Self::from_compressed_bytes_unchecked(compressed)?;

		if unlikely(!header.is_valid()) {
			bail!("header fails validation");
		}
		Ok(header)
	}

	/// Reads a header directly from a file path using memory mapping.
	///
	/// Efficient for quickly inspecting SPZ file metadata without
	/// reading the entire file.
	///
	/// Does NOT validate whether the read header is a valid SPZ header,
	/// simply reads the bytes and interprets them as a header.
	#[inline]
	pub fn from_file_unchecked<P>(spz_path: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		if cfg!(target_os = "macos") {
			// mmap on macos isn't great according to ripgrep code
			let mut input = std::fs::File::open(&spz_path)?;
			let mut buf = [0_u8; COMPRESSED_BLOCK_READ_SIZE as usize];

			input.read_exact(&mut buf)?;

			Self::from_compressed_bytes_unchecked(&buf)
				.with_context(|| "unable to decompress and parse SPZ header")
		} else {
			let mmap = mmap_range(&spz_path, 0, COMPRESSED_BLOCK_READ_SIZE as usize)
				.with_context(|| "unable to memory-map file header range")?;

			if unlikely(mmap.as_ref().len() != COMPRESSED_BLOCK_READ_SIZE as usize) {
				bail!(
					"unable to mmap expected length, expected {} bytes, got {}",
					COMPRESSED_BLOCK_READ_SIZE,
					mmap.as_ref().len()
				);
			}
			Self::from_compressed_bytes_unchecked(mmap.as_ref())
		}
	}

	/// Reads a header directly from a file path using memory mapping.
	///
	/// Memory-maps the file and reads the 1st 16 bytes as a header.
	/// Efficient for quickly inspecting SPZ file metadata without
	/// reading the entire file.
	#[inline]
	pub fn from_file<P>(spz_path: P) -> Result<Self>
	where
		P: AsRef<Path>,
	{
		let ret = Self::from_file_unchecked(spz_path)?;

		if unlikely(!ret.is_valid()) {
			bail!("header fails validation");
		}
		Ok(ret)
	}

	/// Reads a header from the given reader without validation.
	///
	/// Consumes exactly [`HEADER_SIZE`] (16 bytes) from the reader and
	/// interprets them as a header. Does not validate the magic number
	/// or version, for that use a separate validation if needed.
	#[inline]
	pub fn read_from_unchecked<R>(reader: &mut R) -> Result<Self>
	where
		R: Read,
	{
		let mut header_buf = [0; HEADER_SIZE];

		reader.read_exact(&mut header_buf)?;

		header_buf.try_into()
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
		let ret = Self::read_from_unchecked(reader)?;

		if unlikely(!ret.is_valid()) {
			bail!("header fails validation");
		}
		Ok(ret)
	}

	/// Writes this header to the given `stream`.
	///
	/// Writes exactly [`HEADER_SIZE`] (16 bytes) in the binary format
	/// expected by SPZ readers.
	#[inline]
	pub fn serialize_to<W>(&self, stream: &mut W) -> Result<()>
	where
		W: Write,
	{
		let b = unsafe { &*(self as *const Self as *const [u8; HEADER_SIZE]) };

		let n = stream.write(b)?;

		debug_assert_eq!(n, HEADER_SIZE);

		stream.flush()
			.with_context(|| "header serialization: unable to flush stream")
	}

	/// Does some basic validation of this header.
	#[inline]
	pub fn is_valid(&self) -> bool {
		likely(self.magic == MAGIC_VALUE
			&& matches!(self.version, Version::V2 | Version::V3)
			&& (0..=3).contains(&self.spherical_harmonics_degree)
			&& self.num_points >= 0
			&& self.flags.is_valid()
			&& self.reserved == 0)
	}

	pub fn pretty_fmt(&self) -> String {
		use std::fmt::Write;

		let mut ret = String::new();

		let _ = write!(ret, "GaussianSplat Header:\n");
		let _ = write!(ret, "\tNumber of points:\t\t{}\n", self.num_points);
		let _ = write!(
			ret,
			"\tSpherical harmonics degree:\t{}\n",
			self.spherical_harmonics_degree
		);
		let _ = write!(ret, "\tAntialiased:\t\t\t{}\n", self.flags.is_antialiased());

		ret
	}
}

impl std::fmt::Display for Header {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let _ = write!(
			f,
			"Header={{ver={}, n_pts={}, sh_deg={}, fractional_bits={}, antialiased={}}}",
			self.version,
			self.num_points,
			self.spherical_harmonics_degree,
			self.fractional_bits,
			self.flags.is_antialiased(),
		);
		Ok(())
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
			fractional_bits: 12,
			flags: Flags::none(),
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

impl TryFrom<[u8; 16]> for Header {
	type Error = Error;

	#[inline]
	fn try_from(from: [u8; 16]) -> Result<Self> {
		let header = unsafe { std::mem::transmute::<[u8; 16], Self>(from) };

		if unlikely(!header.is_valid()) {
			bail!("header fails validation");
		}
		Ok(header)
	}
}

impl TryFrom<&[u8]> for Header {
	type Error = Error;

	/// Tries to convert the 1st 16 bytes of the given byte slice into a
	/// [`Header`].
	#[inline]
	fn try_from(from: &[u8]) -> Result<Self> {
		if unlikely(from.len() < HEADER_SIZE) {
			bail!(
				"invalid slice length for Header conversion: expected at least 16 bytes, got {} bytes",
				from.len()
			);
		}
		let header = unsafe {
			std::mem::transmute::<[u8; 16], Self>(from[..HEADER_SIZE].try_into()?)
		};
		if unlikely(!header.is_valid()) {
			bail!("header fails validation");
		}
		Ok(header)
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	#[case(Version::V3, 12345, 2, 12, Flags::ANTIALIASED)]
	#[case(Version::V2, 0, 0, 0, Flags::none())]
	#[case(Version::V3, i32::MAX, 3, 24, Flags::none())]
	#[case(Version::V2, 1, 1, 1, Flags::ANTIALIASED)]
	fn test_header_from_array_roundtrip(
		#[case] version: Version,
		#[case] num_points: i32,
		#[case] spherical_harmonics_degree: u8,
		#[case] fractional_bits: u8,
		#[case] flags: Flags,
	) {
		let header = Header {
			magic: MAGIC_VALUE,
			version,
			num_points,
			spherical_harmonics_degree,
			fractional_bits,
			flags,
			..Default::default()
		};
		let bytes: [u8; 16] = header.try_into().expect("should convert");
		let recovered: Header = bytes.try_into().expect("should convert");

		assert_eq!(recovered.magic, MAGIC_VALUE);
		assert_eq!(recovered.version, version);
		assert_eq!(recovered.num_points, num_points);
		assert_eq!(
			recovered.spherical_harmonics_degree,
			spherical_harmonics_degree
		);
		assert_eq!(recovered.fractional_bits, fractional_bits);
		assert_eq!(recovered.flags, flags);
		assert_eq!(recovered.reserved, 0);
	}

	#[rstest]
	#[case(Version::V3, 500, 1, 8)]
	#[case(Version::V2, 1000, 2, 16)]
	#[case(Version::V3, 0, 0, 12)]
	fn test_header_try_from_slice_success(
		#[case] version: Version,
		#[case] num_points: i32,
		#[case] spherical_harmonics_degree: u8,
		#[case] fractional_bits: u8,
	) {
		let header = Header {
			magic: MAGIC_VALUE,
			version,
			num_points,
			spherical_harmonics_degree,
			fractional_bits,
			flags: Flags::none(),
			..Default::default()
		};
		let bytes: [u8; 16] = header.try_into().expect("should convert");
		let recovered = Header::try_from(bytes.as_slice()).expect("should parse");

		assert_eq!(recovered.magic, MAGIC_VALUE);
		assert_eq!(recovered.version, version);
		assert_eq!(recovered.num_points, num_points);
		assert_eq!(
			recovered.spherical_harmonics_degree,
			spherical_harmonics_degree
		);
	}

	#[rstest]
	#[case(100)]
	#[case(1000)]
	fn test_header_try_from_slice_with_extra_bytes(#[case] extra_bytes: usize) {
		let header = Header {
			magic: MAGIC_VALUE,
			version: Version::V2,
			num_points: 999,
			spherical_harmonics_degree: 3,
			fractional_bits: 16,
			flags: Flags::none(),
			..Default::default()
		};
		let bytes: [u8; 16] = header.try_into().expect("should convert");
		let mut extended = bytes.to_vec();

		extended.extend_from_slice(&vec![0xAB; extra_bytes]);

		let recovered = Header::try_from(extended.as_slice()).expect("should parse");

		assert_eq!(recovered.magic, MAGIC_VALUE);
		assert_eq!(recovered.num_points, 999);
	}

	#[rstest]
	#[case(&[0_u8; 15], "expected at least 16 bytes")] // one byte short
	#[case(&[0_u8; 0], "expected at least 16 bytes")] // empty
	#[case(&[0_u8; 8], "expected at least 16 bytes")] // half size
	fn test_header_try_from_slice_errors(#[case] slice: &[u8], #[case] expected_err: &str) {
		let result = Header::try_from(slice);

		assert!(result.is_err());

		let err_msg = result.unwrap_err().to_string();

		assert!(
			err_msg.contains(expected_err),
			"expected '{}' in '{}'",
			expected_err,
			err_msg
		);
	}

	#[rstest]
	#[case(MAGIC_VALUE, Version::V3, 2, 0, true)]
	#[case(MAGIC_VALUE, Version::V2, 0, 0, true)]
	#[case(MAGIC_VALUE, Version::V3, 3, 0, true)]
	#[case(0x12345678, Version::V3, 2, 0, false)] // invalid magic
	#[case(MAGIC_VALUE, Version::V1, 2, 0, false)] // invalid version
	#[case(MAGIC_VALUE, Version::V3, 4, 0, false)] // invalid sh degree
	#[case(MAGIC_VALUE, Version::V3, 2, 1, false)] // invalid reserved
	fn test_header_is_valid(
		#[case] magic: i32,
		#[case] version: Version,
		#[case] sh_degree: u8,
		#[case] reserved: u8,
		#[case] expected_valid: bool,
	) {
		let header = Header {
			magic,
			version,
			num_points: 100,
			spherical_harmonics_degree: sh_degree,
			fractional_bits: 12,
			flags: Flags::none(),
			reserved,
		};
		assert_eq!(header.is_valid(), expected_valid);
	}

	#[rstest]
	#[case(Flags::none(), false, true)]
	#[case(Flags::ANTIALIASED, true, true)]
	#[case(Flags(0x02), false, false)] // undefined bit
	#[case(Flags(0xFF), true, false)] // all bits set — antialiased bit is on but invalid overall
	fn test_flags(
		#[case] flags: Flags,
		#[case] expected_antialiased: bool,
		#[case] expected_valid: bool,
	) {
		assert_eq!(flags.is_antialiased(), expected_antialiased);
		assert_eq!(flags.is_valid(), expected_valid);
	}

	#[test]
	fn test_header_default() {
		let h = Header::default();

		assert_eq!(h.magic, MAGIC_VALUE);
		assert_eq!(h.version, Version::V3);
		assert_eq!(h.num_points, 0);
		assert_eq!(h.spherical_harmonics_degree, 0);
		assert_eq!(h.fractional_bits, 12);
		assert!(!h.flags.is_antialiased());
		assert_eq!(h.reserved, 0);
		assert!(h.is_valid());
	}

	#[test]
	fn test_header_display() {
		let h = Header::default();
		let display = format!("{}", h);

		assert!(display.contains("Header="));
		assert!(display.contains("n_pts=0"));
		assert!(display.contains("v3"));
	}

	#[test]
	fn test_header_pretty_fmt() {
		let h = Header {
			num_points: 42,
			spherical_harmonics_degree: 2,
			flags: Flags::ANTIALIASED,
			..Default::default()
		};
		let pretty = h.pretty_fmt();

		assert!(pretty.contains("42"));
		assert!(pretty.contains("true")); // antialiased
		assert!(pretty.contains("2")); // sh degree
	}

	#[rstest]
	#[case(Version::V1, "v1")]
	#[case(Version::V2, "v2")]
	#[case(Version::V3, "v3")]
	fn test_version_display(#[case] version: Version, #[case] expected: &str) {
		assert_eq!(format!("{}", version), expected);
	}

	#[test]
	fn test_header_negative_num_points_invalid() {
		let h = Header {
			num_points: -1,
			..Default::default()
		};

		assert!(!h.is_valid());
	}

	#[test]
	fn test_serialize_read_roundtrip() {
		let original = Header {
			magic: MAGIC_VALUE,
			version: Version::V3,
			num_points: 1234,
			spherical_harmonics_degree: 3,
			fractional_bits: 16,
			flags: Flags::ANTIALIASED,
			reserved: 0,
		};
		let mut buf = Vec::new();

		original.serialize_to(&mut buf).expect("serialize failed");

		assert_eq!(buf.len(), HEADER_SIZE);

		let mut cursor = std::io::Cursor::new(&buf);
		let recovered = Header::read_from(&mut cursor).expect("read failed");

		assert_eq!(recovered, original);
	}

	#[test]
	fn test_read_from_unchecked_valid_header() {
		let original = Header {
			magic: MAGIC_VALUE,
			version: Version::V2,
			num_points: 500,
			spherical_harmonics_degree: 1,
			fractional_bits: 8,
			flags: Flags::none(),
			reserved: 0,
		};
		let mut buf = Vec::new();

		original.serialize_to(&mut buf).expect("serialize failed");

		let mut cursor = std::io::Cursor::new(&buf);
		let header = Header::read_from_unchecked(&mut cursor)
			.expect("read_from_unchecked failed");

		assert_eq!(header.num_points, 500);
		assert_eq!(header.version, Version::V2);
	}

	#[test]
	fn test_read_from_rejects_invalid_header() {
		let mut bytes = [0_u8; 16];

		bytes[0..4].copy_from_slice(&(0xDEADBEEF_u32 as i32).to_le_bytes());
		bytes[4..8].copy_from_slice(&3_i32.to_le_bytes());

		let mut cursor = std::io::Cursor::new(&bytes);
		let result = Header::read_from(&mut cursor);

		assert!(result.is_err());
	}
}
