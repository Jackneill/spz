// SPDX-License-Identifier: MIT OR Apache-2.0

//! Convenience conversions between `spz` crate types and Cap'n Proto generated
//! types.
//!
//! These conversions are gated behind the `spz` feature flag. They allow
//! ergonomic interop between the native Rust `spz` types and their Cap'n Proto
//! wire-format counterparts.

use capnp::message;
use thiserror::Error;

use crate::spz_capnp;

/// Error returned when a Cap'n Proto [`Version`] value cannot be mapped to a
/// valid [`spz::header::Version`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConvertError {
	/// General error with context. Used for errors that don't fit the other variants.
	#[error("{context}: capnp error: {error}")]
	CapnpErrorWithCtx {
		error: capnp::Error,
		context: String,
	},

	/// The capnp [`Version`] enum has no corresponding [`spz::header::Version`].
	#[error("unsupported capnp version: {0:?}")]
	UnsupportedVersion(spz_capnp::Version),

	/// A Cap'n Proto operation returned an error.
	#[error("capnp error: {0}")]
	CapnpError(#[from] capnp::Error),

	/// A Cap'n Proto enum value was not in the schema.
	#[error("capnp enum not in schema: {0}")]
	NotInSchema(#[from] capnp::NotInSchema),

	/// The `n` value from Cap'n Proto exceeds `i32::MAX`.
	#[error("n value {0} exceeds i32::MAX")]
	NumPointsOverflow(u64),
}

impl From<spz::header::Version> for spz_capnp::Version {
	/// Converts an [`spz::header::Version`] to its Cap'n Proto equivalent.
	///
	/// The discriminant values differ between the two enums:
	/// - `spz`: V1=1, V2=2, V3=3 (repr i32)
	/// - capnp: V1=0, V2=1, V3=2 (repr u16)
	///
	/// Reason: as per according to capnproto doc, capnp enums should not
	/// be considered numeric.
	#[inline]
	fn from(v: spz::header::Version) -> Self {
		match v {
			spz::header::Version::V1 => spz_capnp::Version::V1,
			spz::header::Version::V2 => spz_capnp::Version::V2,
			spz::header::Version::V3 => spz_capnp::Version::V3,
		}
	}
}

impl From<spz_capnp::Version> for spz::header::Version {
	/// Converts a Cap'n Proto [`spz_capnp::Version`] to an
	/// [`spz::header::Version`].
	///
	/// Currently all three capnp variants map directly, but this is
	/// `TryFrom` rather than `From` in case future schema versions
	/// add enumerants that the `spz` crate does not support.
	#[inline]
	fn from(v: spz_capnp::Version) -> Self {
		match v {
			spz_capnp::Version::V1 => spz::header::Version::V1,
			spz_capnp::Version::V2 => spz::header::Version::V2,
			spz_capnp::Version::V3 => spz::header::Version::V3,
		}
	}
}

impl From<spz::coord::CoordinateSystem> for spz_capnp::CoordinateSystem {
	/// Converts an [`spz::coord::CoordinateSystem`] to its Cap'n Proto
	/// equivalent. Variant values are 1:1.
	#[inline]
	fn from(cs: spz::coord::CoordinateSystem) -> Self {
		match cs {
			spz::coord::CoordinateSystem::Unspecified => {
				spz_capnp::CoordinateSystem::Unspecified
			},
			spz::coord::CoordinateSystem::LeftDownBack => {
				spz_capnp::CoordinateSystem::LeftDownBack
			},
			spz::coord::CoordinateSystem::RightDownBack => {
				spz_capnp::CoordinateSystem::RightDownBack
			},
			spz::coord::CoordinateSystem::LeftUpBack => {
				spz_capnp::CoordinateSystem::LeftUpBack
			},
			spz::coord::CoordinateSystem::RightUpBack => {
				spz_capnp::CoordinateSystem::RightUpBack
			},
			spz::coord::CoordinateSystem::LeftDownFront => {
				spz_capnp::CoordinateSystem::LeftDownFront
			},
			spz::coord::CoordinateSystem::RightDownFront => {
				spz_capnp::CoordinateSystem::RightDownFront
			},
			spz::coord::CoordinateSystem::LeftUpFront => {
				spz_capnp::CoordinateSystem::LeftUpFront
			},
			spz::coord::CoordinateSystem::RightUpFront => {
				spz_capnp::CoordinateSystem::RightUpFront
			},
		}
	}
}

impl From<spz_capnp::CoordinateSystem> for spz::coord::CoordinateSystem {
	/// Converts a Cap'n Proto [`spz_capnp::CoordinateSystem`] to an
	/// [`spz::coord::CoordinateSystem`]. Variant values are 1:1.
	#[inline]
	fn from(cs: spz_capnp::CoordinateSystem) -> Self {
		match cs {
			spz_capnp::CoordinateSystem::Unspecified => {
				spz::coord::CoordinateSystem::Unspecified
			},
			spz_capnp::CoordinateSystem::LeftDownBack => {
				spz::coord::CoordinateSystem::LeftDownBack
			},
			spz_capnp::CoordinateSystem::RightDownBack => {
				spz::coord::CoordinateSystem::RightDownBack
			},
			spz_capnp::CoordinateSystem::LeftUpBack => {
				spz::coord::CoordinateSystem::LeftUpBack
			},
			spz_capnp::CoordinateSystem::RightUpBack => {
				spz::coord::CoordinateSystem::RightUpBack
			},
			spz_capnp::CoordinateSystem::LeftDownFront => {
				spz::coord::CoordinateSystem::LeftDownFront
			},
			spz_capnp::CoordinateSystem::RightDownFront => {
				spz::coord::CoordinateSystem::RightDownFront
			},
			spz_capnp::CoordinateSystem::LeftUpFront => {
				spz::coord::CoordinateSystem::LeftUpFront
			},
			spz_capnp::CoordinateSystem::RightUpFront => {
				spz::coord::CoordinateSystem::RightUpFront
			},
		}
	}
}

/// Writes an [`spz::header::Header`] into a Cap'n Proto
/// [`header::Builder`](spz_capnp::header::Builder).
///
/// The `magic` and `reserved` fields from the spz header are intentionally
/// omitted from the Cap'n Proto schema because they are constants.
pub fn write_header_to_builder(
	src: &spz::header::Header,
	dst: &mut spz_capnp::header::Builder<'_>,
) {
	dst.set_version(spz_capnp::Version::from(src.version));
	dst.set_n(src.num_points as u64);
	dst.set_spherical_harmonics_degree(src.spherical_harmonics_degree);
	dst.set_fractional_bits(src.fractional_bits);
	dst.set_flags(src.flags.bits());
}

/// Reads a Cap'n Proto [`header::Reader`](spz_capnp::header::Reader) into an
/// [`spz::header::Header`].
///
/// The `magic` field is populated with the SPZ magic constant and `reserved`
/// is set to `0`.
pub fn read_header_from_reader(
	reader: spz_capnp::header::Reader<'_>,
) -> Result<spz::header::Header, ConvertError> {
	let capnp_version = reader.get_version()?;
	let version = spz::header::Version::from(capnp_version);
	let n = reader.get_n();

	if n > i32::MAX as u64 {
		return Err(ConvertError::NumPointsOverflow(n));
	}
	Ok(spz::header::Header {
		magic: spz::header::MAGIC_VALUE,
		version,
		num_points: n as i32,
		spherical_harmonics_degree: reader.get_spherical_harmonics_degree(),
		fractional_bits: reader.get_fractional_bits(),
		flags: spz::header::Flags(reader.get_flags()),
		reserved: 0,
	})
}

/// Writes an [`spz::gaussian_splat::GaussianSplat`] into a Cap'n Proto
/// message and returns the serialised [`message::Builder`].
///
/// This populates both the `header` and `body` sub-messages.
pub fn gaussian_splat_to_message(
	src: &spz::gaussian_splat::GaussianSplat,
) -> message::Builder<message::HeapAllocator> {
	let mut msg = message::Builder::new_default();

	{
		let mut root = msg.init_root::<spz_capnp::gaussian_splat::Builder<'_>>();
		let mut hdr = root.reborrow().init_header();

		write_header_to_builder(&src.header, &mut hdr);

		let mut body = root.init_body();

		set_gaussian_splat_body_f32_list(&mut body, BodyField::Positions, &src.positions);
		set_gaussian_splat_body_f32_list(&mut body, BodyField::Scales, &src.scales);
		set_gaussian_splat_body_f32_list(&mut body, BodyField::Rotations, &src.rotations);
		set_gaussian_splat_body_f32_list(&mut body, BodyField::Alphas, &src.alphas);
		set_gaussian_splat_body_f32_list(&mut body, BodyField::Colors, &src.colors);
		set_gaussian_splat_body_f32_list(
			&mut body,
			BodyField::SphericalHarmonics,
			&src.spherical_harmonics,
		);
	}

	msg
}

/// Reads a [`spz::gaussian_splat::GaussianSplat`] from a Cap'n Proto
/// [`gaussian_splat::Reader`](spz_capnp::gaussian_splat::Reader).
pub fn gaussian_splat_from_reader(
	reader: spz_capnp::gaussian_splat::Reader<'_>,
) -> Result<spz::gaussian_splat::GaussianSplat, ConvertError> {
	let hdr_reader = reader.get_header()?;
	let header = read_header_from_reader(hdr_reader)?;

	let body = reader.get_body()?;

	let positions = read_f32_list(body.get_positions()?);
	let scales = read_f32_list(body.get_scales()?);
	let rotations = read_f32_list(body.get_rotations()?);
	let alphas = read_f32_list(body.get_alphas()?);
	let colors = read_f32_list(body.get_colors()?);
	let spherical_harmonics = read_f32_list(body.get_spherical_harmonics()?);

	Ok(spz::gaussian_splat::GaussianSplat {
		header,
		positions,
		scales,
		rotations,
		alphas,
		colors,
		spherical_harmonics,
	})
}

/// Serializes an [`spz::gaussian_splat::GaussianSplat`] to Cap'n Proto wire
/// format bytes.
pub fn serialize_to_bytes(
	splat: &spz::gaussian_splat::GaussianSplat,
) -> Result<Vec<u8>, ConvertError> {
	let msg = gaussian_splat_to_message(splat);
	let mut buf = Vec::new();

	capnp::serialize::write_message(&mut buf, &msg).map_err(|e| {
		ConvertError::CapnpErrorWithCtx {
			error: e,
			context: "unable to serialize GaussianSplat to bytes".to_string(),
		}
	})?;
	Ok(buf)
}

/// Deserializes an [`spz::gaussian_splat::GaussianSplat`] from Cap'n Proto
/// wire format bytes.
pub fn deserialize_from_bytes(
	bytes: &[u8],
) -> Result<spz::gaussian_splat::GaussianSplat, ConvertError> {
	let reader = capnp::serialize::read_message(bytes, message::ReaderOptions::default())?;
	let root = reader.get_root::<spz_capnp::gaussian_splat::Reader<'_>>()?;

	gaussian_splat_from_reader(root)
}

/// Which body field to set. Avoids duplicating the init+copy pattern six times.
enum BodyField {
	Positions,
	Scales,
	Rotations,
	Alphas,
	Colors,
	SphericalHarmonics,
}

fn set_gaussian_splat_body_f32_list(
	body: &mut spz_capnp::body::Builder<'_>,
	field: BodyField,
	data: &[f32],
) {
	let len = data.len() as u32;
	let mut list = match field {
		BodyField::Positions => body.reborrow().init_positions(len),
		BodyField::Scales => body.reborrow().init_scales(len),
		BodyField::Rotations => body.reborrow().init_rotations(len),
		BodyField::Alphas => body.reborrow().init_alphas(len),
		BodyField::Colors => body.reborrow().init_colors(len),
		BodyField::SphericalHarmonics => body.reborrow().init_spherical_harmonics(len),
	};
	for (i, &val) in data.iter().enumerate() {
		list.set(i as u32, val);
	}
}

fn read_f32_list(list: capnp::primitive_list::Reader<'_, f32>) -> Vec<f32> {
	(0..list.len()).map(|i| list.get(i)).collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	use rstest::rstest;

	#[rstest]
	#[case::v1(spz::header::Version::V1, spz_capnp::Version::V1)]
	#[case::v2(spz::header::Version::V2, spz_capnp::Version::V2)]
	#[case::v3(spz::header::Version::V3, spz_capnp::Version::V3)]
	fn version_spz_to_capnp(
		#[case] spz_ver: spz::header::Version,
		#[case] expected: spz_capnp::Version,
	) {
		assert_eq!(spz_capnp::Version::from(spz_ver), expected);
	}

	#[rstest]
	#[case::v1(spz_capnp::Version::V1, spz::header::Version::V1)]
	#[case::v2(spz_capnp::Version::V2, spz::header::Version::V2)]
	#[case::v3(spz_capnp::Version::V3, spz::header::Version::V3)]
	fn version_capnp_to_spz(
		#[case] capnp_ver: spz_capnp::Version,
		#[case] expected: spz::header::Version,
	) {
		assert_eq!(spz::header::Version::try_from(capnp_ver).unwrap(), expected);
	}

	#[rstest]
	#[case::v1(spz::header::Version::V1)]
	#[case::v2(spz::header::Version::V2)]
	#[case::v3(spz::header::Version::V3)]
	fn version_roundtrip(#[case] original: spz::header::Version) {
		let capnp_ver = spz_capnp::Version::from(original);
		let back = spz::header::Version::try_from(capnp_ver).unwrap();
		assert_eq!(original, back);
	}

	#[rstest]
	#[case::unspecified(
		spz::coord::CoordinateSystem::Unspecified,
		spz_capnp::CoordinateSystem::Unspecified
	)]
	#[case::ldb(
		spz::coord::CoordinateSystem::LeftDownBack,
		spz_capnp::CoordinateSystem::LeftDownBack
	)]
	#[case::rdb(
		spz::coord::CoordinateSystem::RightDownBack,
		spz_capnp::CoordinateSystem::RightDownBack
	)]
	#[case::lub(
		spz::coord::CoordinateSystem::LeftUpBack,
		spz_capnp::CoordinateSystem::LeftUpBack
	)]
	#[case::rub(
		spz::coord::CoordinateSystem::RightUpBack,
		spz_capnp::CoordinateSystem::RightUpBack
	)]
	#[case::ldf(
		spz::coord::CoordinateSystem::LeftDownFront,
		spz_capnp::CoordinateSystem::LeftDownFront
	)]
	#[case::rdf(
		spz::coord::CoordinateSystem::RightDownFront,
		spz_capnp::CoordinateSystem::RightDownFront
	)]
	#[case::luf(
		spz::coord::CoordinateSystem::LeftUpFront,
		spz_capnp::CoordinateSystem::LeftUpFront
	)]
	#[case::ruf(
		spz::coord::CoordinateSystem::RightUpFront,
		spz_capnp::CoordinateSystem::RightUpFront
	)]
	fn coordinate_system_spz_to_capnp(
		#[case] spz_cs: spz::coord::CoordinateSystem,
		#[case] expected: spz_capnp::CoordinateSystem,
	) {
		assert_eq!(spz_capnp::CoordinateSystem::from(spz_cs), expected);
	}

	#[rstest]
	#[case::unspecified(
		spz_capnp::CoordinateSystem::Unspecified,
		spz::coord::CoordinateSystem::Unspecified
	)]
	#[case::ldb(
		spz_capnp::CoordinateSystem::LeftDownBack,
		spz::coord::CoordinateSystem::LeftDownBack
	)]
	#[case::rdb(
		spz_capnp::CoordinateSystem::RightDownBack,
		spz::coord::CoordinateSystem::RightDownBack
	)]
	#[case::lub(
		spz_capnp::CoordinateSystem::LeftUpBack,
		spz::coord::CoordinateSystem::LeftUpBack
	)]
	#[case::rub(
		spz_capnp::CoordinateSystem::RightUpBack,
		spz::coord::CoordinateSystem::RightUpBack
	)]
	#[case::ldf(
		spz_capnp::CoordinateSystem::LeftDownFront,
		spz::coord::CoordinateSystem::LeftDownFront
	)]
	#[case::rdf(
		spz_capnp::CoordinateSystem::RightDownFront,
		spz::coord::CoordinateSystem::RightDownFront
	)]
	#[case::luf(
		spz_capnp::CoordinateSystem::LeftUpFront,
		spz::coord::CoordinateSystem::LeftUpFront
	)]
	#[case::ruf(
		spz_capnp::CoordinateSystem::RightUpFront,
		spz::coord::CoordinateSystem::RightUpFront
	)]
	fn coordinate_system_capnp_to_spz(
		#[case] capnp_cs: spz_capnp::CoordinateSystem,
		#[case] expected: spz::coord::CoordinateSystem,
	) {
		assert_eq!(spz::coord::CoordinateSystem::from(capnp_cs), expected);
	}

	#[rstest]
	#[case::unspecified(spz::coord::CoordinateSystem::Unspecified)]
	#[case::ldb(spz::coord::CoordinateSystem::LeftDownBack)]
	#[case::rdb(spz::coord::CoordinateSystem::RightDownBack)]
	#[case::lub(spz::coord::CoordinateSystem::LeftUpBack)]
	#[case::rub(spz::coord::CoordinateSystem::RightUpBack)]
	#[case::ldf(spz::coord::CoordinateSystem::LeftDownFront)]
	#[case::rdf(spz::coord::CoordinateSystem::RightDownFront)]
	#[case::luf(spz::coord::CoordinateSystem::LeftUpFront)]
	#[case::ruf(spz::coord::CoordinateSystem::RightUpFront)]
	fn coordinate_system_roundtrip(#[case] original: spz::coord::CoordinateSystem) {
		let capnp_cs = spz_capnp::CoordinateSystem::from(original);
		let back = spz::coord::CoordinateSystem::from(capnp_cs);
		assert_eq!(original, back);
	}

	fn make_test_header() -> spz::header::Header {
		spz::header::Header {
			magic: spz::header::MAGIC_VALUE,
			version: spz::header::Version::V3,
			num_points: 42,
			spherical_harmonics_degree: 2,
			fractional_bits: 12,
			flags: spz::header::Flags::ANTIALIASED,
			reserved: 0,
		}
	}

	#[test]
	fn header_roundtrip_via_builder() {
		let original = make_test_header();

		// Write to a capnp message.
		let mut msg = message::Builder::new_default();
		{
			let mut builder = msg.init_root::<spz_capnp::header::Builder<'_>>();

			write_header_to_builder(&original, &mut builder);
		}
		// Read back.
		let reader = msg
			.get_root_as_reader::<spz_capnp::header::Reader<'_>>()
			.unwrap();
		let restored = read_header_from_reader(reader).unwrap();

		assert_eq!(original.version, restored.version);
		assert_eq!(original.num_points, restored.num_points);
		assert_eq!(
			original.spherical_harmonics_degree,
			restored.spherical_harmonics_degree
		);
		assert_eq!(original.fractional_bits, restored.fractional_bits);
		assert_eq!(original.flags, restored.flags);
		assert_eq!(restored.magic, spz::header::MAGIC_VALUE);
		assert_eq!(restored.reserved, 0);
	}

	#[rstest]
	#[case::v2(spz::header::Version::V2)]
	#[case::v3(spz::header::Version::V3)]
	fn header_roundtrip_versions(#[case] version: spz::header::Version) {
		let original = spz::header::Header {
			version,
			..make_test_header()
		};
		let mut msg = message::Builder::new_default();
		{
			let mut builder = msg.init_root::<spz_capnp::header::Builder<'_>>();

			write_header_to_builder(&original, &mut builder);
		}
		let reader = msg
			.get_root_as_reader::<spz_capnp::header::Reader<'_>>()
			.unwrap();
		let restored = read_header_from_reader(reader).unwrap();

		assert_eq!(original.version, restored.version);
	}

	#[test]
	fn header_num_points_overflow() {
		let mut msg = message::Builder::new_default();
		{
			let mut builder = msg.init_root::<spz_capnp::header::Builder<'_>>();

			builder.set_version(spz_capnp::Version::V3);
			builder.set_n(i32::MAX as u64 + 1);
			builder.set_spherical_harmonics_degree(0);
			builder.set_fractional_bits(12);
			builder.set_flags(0);
		}
		let reader = msg
			.get_root_as_reader::<spz_capnp::header::Reader<'_>>()
			.unwrap();
		let err = read_header_from_reader(reader).unwrap_err();

		assert!(
			matches!(err, ConvertError::NumPointsOverflow(_)),
			"expected NumPointsOverflow, got: {err:?}"
		);
	}

	#[rstest]
	#[case::zero_points(0)]
	#[case::one_point(1)]
	#[case::many_points(100_000)]
	#[case::max_i32(i32::MAX)]
	fn header_various_num_points(#[case] n: i32) {
		let original = spz::header::Header {
			num_points: n,
			..make_test_header()
		};
		let mut msg = message::Builder::new_default();
		{
			let mut builder = msg.init_root::<spz_capnp::header::Builder<'_>>();

			write_header_to_builder(&original, &mut builder);
		}
		let reader = msg
			.get_root_as_reader::<spz_capnp::header::Reader<'_>>()
			.unwrap();
		let restored = read_header_from_reader(reader).unwrap();

		assert_eq!(original.num_points, restored.num_points);
	}

	#[rstest]
	#[case::no_flags(spz::header::Flags::none())]
	#[case::antialiased(spz::header::Flags::ANTIALIASED)]
	fn header_flags_roundtrip(#[case] flags: spz::header::Flags) {
		let original = spz::header::Header {
			flags,
			..make_test_header()
		};
		let mut msg = message::Builder::new_default();
		{
			let mut builder = msg.init_root::<spz_capnp::header::Builder<'_>>();

			write_header_to_builder(&original, &mut builder);
		}
		let reader = msg
			.get_root_as_reader::<spz_capnp::header::Reader<'_>>()
			.unwrap();
		let restored = read_header_from_reader(reader).unwrap();

		assert_eq!(original.flags, restored.flags);
	}

	fn make_test_splat() -> spz::gaussian_splat::GaussianSplat {
		spz::gaussian_splat::GaussianSplat {
			header: spz::header::Header {
				magic: spz::header::MAGIC_VALUE,
				version: spz::header::Version::V3,
				num_points: 2,
				spherical_harmonics_degree: 0,
				fractional_bits: 12,
				flags: spz::header::Flags::none(),
				reserved: 0,
			},
			// 2 points × 3 components
			positions: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
			scales: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
			rotations: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
			alphas: vec![0.5, 0.9],
			colors: vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
			spherical_harmonics: vec![],
		}
	}

	#[test]
	fn gaussian_splat_message_roundtrip() {
		let original = make_test_splat();
		let msg = gaussian_splat_to_message(&original);

		let reader = msg
			.get_root_as_reader::<spz_capnp::gaussian_splat::Reader<'_>>()
			.unwrap();
		let restored = gaussian_splat_from_reader(reader).unwrap();

		assert_eq!(original.header.version, restored.header.version);
		assert_eq!(original.header.num_points, restored.header.num_points);
		assert_eq!(original.positions, restored.positions);
		assert_eq!(original.scales, restored.scales);
		assert_eq!(original.rotations, restored.rotations);
		assert_eq!(original.alphas, restored.alphas);
		assert_eq!(original.colors, restored.colors);
		assert_eq!(original.spherical_harmonics, restored.spherical_harmonics);
	}

	#[test]
	fn gaussian_splat_serialize_deserialize_roundtrip() {
		let original = make_test_splat();
		let bytes = serialize_to_bytes(&original).expect("unable to serialize to bytes");

		assert!(!bytes.is_empty(), "serialized bytes should not be empty");

		let restored = deserialize_from_bytes(&bytes).unwrap();

		assert_eq!(original.header.version, restored.header.version);
		assert_eq!(original.header.num_points, restored.header.num_points);
		assert_eq!(
			original.header.spherical_harmonics_degree,
			restored.header.spherical_harmonics_degree
		);
		assert_eq!(
			original.header.fractional_bits,
			restored.header.fractional_bits
		);
		assert_eq!(original.header.flags, restored.header.flags);
		assert_eq!(original.positions, restored.positions);
		assert_eq!(original.scales, restored.scales);
		assert_eq!(original.rotations, restored.rotations);
		assert_eq!(original.alphas, restored.alphas);
		assert_eq!(original.colors, restored.colors);
		assert_eq!(original.spherical_harmonics, restored.spherical_harmonics);
	}

	#[test]
	fn gaussian_splat_empty_body() {
		let splat = spz::gaussian_splat::GaussianSplat {
			header: spz::header::Header {
				magic: spz::header::MAGIC_VALUE,
				version: spz::header::Version::V3,
				num_points: 0,
				spherical_harmonics_degree: 0,
				fractional_bits: 12,
				flags: spz::header::Flags::none(),
				reserved: 0,
			},
			positions: vec![],
			scales: vec![],
			rotations: vec![],
			alphas: vec![],
			colors: vec![],
			spherical_harmonics: vec![],
		};

		let bytes = serialize_to_bytes(&splat).expect("unable to serialize to bytes");
		let restored =
			deserialize_from_bytes(&bytes).expect("unable to deserialize from bytes");

		assert_eq!(restored.header.num_points, 0);
		assert!(restored.positions.is_empty());
		assert!(restored.scales.is_empty());
		assert!(restored.rotations.is_empty());
		assert!(restored.alphas.is_empty());
		assert!(restored.colors.is_empty());
		assert!(restored.spherical_harmonics.is_empty());
	}

	#[test]
	fn gaussian_splat_with_spherical_harmonics() {
		let splat = spz::gaussian_splat::GaussianSplat {
			header: spz::header::Header {
				magic: spz::header::MAGIC_VALUE,
				version: spz::header::Version::V3,
				num_points: 1,
				spherical_harmonics_degree: 1,
				fractional_bits: 12,
				flags: spz::header::Flags::ANTIALIASED,
				reserved: 0,
			},
			positions: vec![1.0, 2.0, 3.0],
			scales: vec![0.1, 0.2, 0.3],
			rotations: vec![0.0, 0.0, 0.0, 1.0],
			alphas: vec![1.0],
			colors: vec![0.5, 0.5, 0.5],
			// degree 1 = 9 coefficients per gaussian
			spherical_harmonics: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9],
		};

		let bytes = serialize_to_bytes(&splat).expect("unable to serialize to bytes");
		let restored =
			deserialize_from_bytes(&bytes).expect("unable to deserialize from bytes");

		assert_eq!(restored.header.spherical_harmonics_degree, 1);
		assert_eq!(restored.spherical_harmonics.len(), 9);
		assert_eq!(splat.spherical_harmonics, restored.spherical_harmonics);
	}

	#[rstest]
	#[case::degree_0(0, 0)]
	#[case::degree_1(1, 9)]
	#[case::degree_2(2, 24)]
	#[case::degree_3(3, 45)]
	fn gaussian_splat_sh_degrees(#[case] degree: u8, #[case] expected_sh_len: usize) {
		let sh_data: Vec<f32> = (0..expected_sh_len).map(|i| i as f32 * 0.01).collect();

		let splat = spz::gaussian_splat::GaussianSplat {
			header: spz::header::Header {
				magic: spz::header::MAGIC_VALUE,
				version: spz::header::Version::V3,
				num_points: 1,
				spherical_harmonics_degree: degree,
				fractional_bits: 12,
				flags: spz::header::Flags::none(),
				reserved: 0,
			},
			positions: vec![0.0; 3],
			scales: vec![0.0; 3],
			rotations: vec![0.0; 4],
			alphas: vec![0.0],
			colors: vec![0.0; 3],
			spherical_harmonics: sh_data.clone(),
		};

		let bytes = serialize_to_bytes(&splat).expect("unable to serialize to bytes");
		let restored =
			deserialize_from_bytes(&bytes).expect("unable to deserialize from bytes");

		assert_eq!(restored.header.spherical_harmonics_degree, degree);
		assert_eq!(restored.spherical_harmonics, sh_data);
	}

	// -- Capnp serialization (no conversion) --------------------------------

	#[test]
	fn capnp_header_builder_reader_basic() {
		let mut msg = message::Builder::new_default();
		{
			let mut hdr = msg.init_root::<spz_capnp::header::Builder<'_>>();

			hdr.set_version(spz_capnp::Version::V3);
			hdr.set_n(100);
			hdr.set_spherical_harmonics_degree(2);
			hdr.set_fractional_bits(12);
			hdr.set_flags(0x01);
		}
		let reader = msg
			.get_root_as_reader::<spz_capnp::header::Reader<'_>>()
			.unwrap();

		assert_eq!(reader.get_version().unwrap(), spz_capnp::Version::V3);
		assert_eq!(reader.get_n(), 100);
		assert_eq!(reader.get_spherical_harmonics_degree(), 2);
		assert_eq!(reader.get_fractional_bits(), 12);
		assert_eq!(reader.get_flags(), 0x01);
	}

	#[test]
	fn capnp_body_builder_reader_basic() {
		let mut msg = message::Builder::new_default();
		{
			let mut body = msg.init_root::<spz_capnp::body::Builder<'_>>();
			let mut pos = body.reborrow().init_positions(3);

			pos.set(0, 1.0);
			pos.set(1, 2.0);
			pos.set(2, 3.0);

			let mut alphas = body.reborrow().init_alphas(1);

			alphas.set(0, 0.5);
		}

		let reader = msg
			.get_root_as_reader::<spz_capnp::body::Reader<'_>>()
			.unwrap();

		let positions = reader.get_positions().unwrap();

		assert_eq!(positions.len(), 3);
		assert_eq!(positions.get(0), 1.0);
		assert_eq!(positions.get(1), 2.0);
		assert_eq!(positions.get(2), 3.0);

		let alphas = reader.get_alphas().unwrap();

		assert_eq!(alphas.len(), 1);
		assert_eq!(alphas.get(0), 0.5);
	}

	#[test]
	fn capnp_gaussian_splat_builder_reader_basic() {
		let mut msg = message::Builder::new_default();
		{
			let mut root = msg.init_root::<spz_capnp::gaussian_splat::Builder<'_>>();
			let mut hdr = root.reborrow().init_header();

			hdr.set_version(spz_capnp::Version::V2);
			hdr.set_n(1);
			hdr.set_spherical_harmonics_degree(0);
			hdr.set_fractional_bits(12);
			hdr.set_flags(0);

			let mut body = root.init_body();
			let mut pos = body.reborrow().init_positions(3);

			pos.set(0, 10.0);
			pos.set(1, 20.0);
			pos.set(2, 30.0);
		}

		let reader = msg
			.get_root_as_reader::<spz_capnp::gaussian_splat::Reader<'_>>()
			.unwrap();

		assert!(reader.has_header());
		assert!(reader.has_body());

		let hdr = reader.get_header().unwrap();

		assert_eq!(hdr.get_version().unwrap(), spz_capnp::Version::V2);
		assert_eq!(hdr.get_n(), 1);

		let body = reader.get_body().unwrap();
		let positions = body.get_positions().unwrap();

		assert_eq!(positions.len(), 3);
		assert_eq!(positions.get(0), 10.0);
	}

	#[test]
	fn capnp_version_try_from_invalid() {
		let result = spz_capnp::Version::try_from(99_u16);

		assert!(result.is_err());
	}

	#[test]
	fn capnp_coordinate_system_try_from_invalid() {
		let result = spz_capnp::CoordinateSystem::try_from(99_u16);

		assert!(result.is_err());
	}

	#[test]
	fn capnp_empty_message_has_defaults() {
		let mut msg = message::Builder::new_default();
		{
			let _hdr = msg.init_root::<spz_capnp::header::Builder<'_>>();
			// Don't set anything — check defaults.
		}
		let reader = msg
			.get_root_as_reader::<spz_capnp::header::Reader<'_>>()
			.unwrap();

		// capnp default for u16 enum is 0 → V1
		assert_eq!(reader.get_version().unwrap(), spz_capnp::Version::V1);
		assert_eq!(reader.get_n(), 0);
		assert_eq!(reader.get_spherical_harmonics_degree(), 0);
		assert_eq!(reader.get_fractional_bits(), 0);
		assert_eq!(reader.get_flags(), 0);
	}

	#[test]
	fn capnp_wire_format_roundtrip() {
		let mut msg = message::Builder::new_default();
		{
			let mut hdr = msg.init_root::<spz_capnp::header::Builder<'_>>();

			hdr.set_version(spz_capnp::Version::V3);
			hdr.set_n(999);
			hdr.set_spherical_harmonics_degree(3);
			hdr.set_fractional_bits(8);
			hdr.set_flags(0x01);
		}
		// Serialize to bytes.
		let mut buf = Vec::new();

		capnp::serialize::write_message(&mut buf, &msg).unwrap();

		// Deserialize from bytes.
		let reader_msg = capnp::serialize::read_message(
			buf.as_slice(),
			message::ReaderOptions::default(),
		)
		.unwrap();

		let reader = reader_msg
			.get_root::<spz_capnp::header::Reader<'_>>()
			.unwrap();

		assert_eq!(reader.get_version().unwrap(), spz_capnp::Version::V3);
		assert_eq!(reader.get_n(), 999);
		assert_eq!(reader.get_spherical_harmonics_degree(), 3);
		assert_eq!(reader.get_fractional_bits(), 8);
		assert_eq!(reader.get_flags(), 0x01);
	}
}
