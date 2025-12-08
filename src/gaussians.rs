use std::io::{BufReader, Write};

use anyhow::Result;

use crate::types::CoordinateConverter;
use crate::types::UnpackOptions;
use crate::{gaussian_cloud::GaussianCloud, types::PackedGaussiansHeader};

use std::io::Read;

use anyhow::bail;

use crate::{consts, util};

/// Represents a single inflated gaussian.
///
/// Each gaussian has 236 bytes.
/// Although the data is easier to intepret in this format,
/// it is not more precise than the packed format, since it was inflated.
#[derive(Clone, Debug, Default)]
pub struct UnpackedGaussian {
	position: [f64; 3], // x, y, z
	rotation: [f64; 4], // x, y, z, w
	scale: [f64; 3],    // std::log(scale)
	color: [f64; 3],    // rgb sh0 encoding
	alpha: f64,         // inverse logistic
	sh_r: [f64; 15],
	sh_g: [f64; 15],
	sh_b: [f64; 15],
}

/// Represents a single low precision gaussian.
///
/// Each gaussian has exactly 65 bytes, even if it does not have full spherical
/// harmonics.
#[derive(Clone, Debug, Default)]
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
	pub fn unpack(
		&self,
		_uses_float16: bool,
		_uses_quaternion_smallest_three: bool,
		_fractional_bits: i32,
		_coord_conv: &CoordinateConverter,
	) -> UnpackedGaussian {
		// TODO: implement unpack logic using project's quantization/coordinate conversion rules
		unimplemented!()
	}
}

/// Represents a full splat with lower precision.
///
/// Each splat has at most 64 bytes, although splats with fewer spherical
/// harmonics degrees will have less.
/// The data is stored non-interleaved.
#[derive(Clone, Debug, Default)]
pub struct PackedGaussians {
	// Total number of points (gaussians)
	pub num_points: i32,
	// Degree of spherical harmonics
	pub sh_degree: i32,
	// Number of bits used for fractional part of fixed-point coords
	pub fractional_bits: i32,
	// Whether gaussians should be rendered with mip-splat antialiasing
	pub antialiased: bool,
	// Whether gaussians use the smallest three method to store quaternions
	pub uses_quaternion_smallest_three: bool,

	pub positions: Vec<u8>,
	pub scales: Vec<u8>,
	pub rotations: Vec<u8>,
	pub alphas: Vec<u8>,
	pub colors: Vec<u8>,
	pub sh: Vec<u8>,
}

impl PackedGaussians {
	pub fn serialize_packed_gaussians(&self, _out: &mut dyn Write) -> Result<()> {
		// Write packed representation to an io::Write stream
		unimplemented!()
	}

	pub fn uses_float16(&self) -> bool {
		// Determine based on packed format metadata. Placeholder.
		unimplemented!()
	}

	pub fn at(&self, _i: i32) -> PackedGaussian {
		// Extract the i-th packed gaussian from interleaved/non-interleaved buffers.
		unimplemented!()
	}

	pub fn unpack(&self, i: i32, coord_conv: &CoordinateConverter) -> UnpackedGaussian {
		let pg = self.at(i);

		pg.unpack(
			self.uses_float16(),
			self.uses_quaternion_smallest_three,
			self.fractional_bits,
			coord_conv,
		)
	}
}

impl TryFrom<String> for PackedGaussians {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		Self::try_from(s.as_str())
	}
}

impl TryFrom<&str> for PackedGaussians {
	type Error = anyhow::Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		const MAX_POINTS_TO_READ: u32 = 10_000_000;
		const HEADER_SIZE: usize = std::mem::size_of::<PackedGaussiansHeader>();

		let from_reader = BufReader::new(s.as_bytes());
		let header: PackedGaussiansHeader = {
			let header_buf: [u8; HEADER_SIZE] = [0; HEADER_SIZE];

			match from_reader.read_exact(&mut header_buf) {
				Ok(_) => {},
				Err(err) => {
					bail!(err);
				},
			}
			unsafe {
				std::ptr::read_unaligned(
					header_buf.as_ptr() as *const PackedGaussiansHeader
				)
			}
		};
		if header.magic != consts::PACKED_GAUSSIAN_HEADER_MAGIC {
			bail!("invalid magic number in packed gaussians header");
		}
		if header.version < 1 || header.version > 3 {
			bail!("unsupported version: {}", header.version);
		}
		if header.num_points > MAX_POINTS_TO_READ {
			bail!(
				"number of points: {} exceeds maximum allowed: {}",
				header.num_points,
				MAX_POINTS_TO_READ
			);
		}
		if header.sh_degree > 3 {
			bail!("unsupported SH degree: {}", header.sh_degree);
		}
		let num_points = header.num_points;
		let sh_dim = util::dim_for_degree(header.sh_degree);
		let uses_float16 = header.version == 1;
		let uses_quaternion_smallest_three = header.version >= 3;

		let mut result = PackedGaussians::default();

		result.num_points = num_points;
		result.sh_degree = header.sh_degree;
		result.fractional_bits = header.fractional_bits;
		result.antialiased = (header.flags & consts::FLAG_ANTIALIASED) != 0;

		// Resize buffers to expected sizes
		let pos_components = 3 * if uses_float16 { 2 } else { 3 };

		result.positions
			.resize((num_points as usize) * pos_components);
		result.scales.resize((num_points as usize) * 3);
		result.uses_quaternion_smallest_three = uses_quaternion_smallest_three;
		result.rotations
			.resize((num_points as usize)
				* if uses_quaternion_smallest_three { 4 } else { 3 });
		result.alphas.resize(num_points as usize);
		result.colors.resize((num_points as usize) * 3);
		result.sh.resize((num_points as usize) * sh_dim * 3);

		// Helper to read exact number of bytes into a u8 vec
		let mut read_buf = |buf: &mut [u8]| -> bool { from_reader.read_exact(buf).is_ok() };

		if !read_buf(&mut result.positions) {
			bail!("read error (positions)");
		}
		if !read_buf(&mut result.alphas) {
			bail!("read error (alphas)");
		}
		if !read_buf(&mut result.colors) {
			bail!("read error (colors)");
		}
		if !read_buf(&mut result.scales) {
			bail!("read error (scales)");
		}
		if !read_buf(&mut result.rotations) {
			bail!("read error (rotations)");
		}
		if !read_buf(&mut result.sh) {
			bail!("read error (sh)");
		}
		Ok(result)
	}
}

// Packed loaders (file and memory variants)
pub fn load_spz_packed_from_file(filename: &str) -> PackedGaussians {
	// Read file and parse to PackedGaussians
	unimplemented!()
}

pub fn load_spz_packed_from_ptr(data: *const u8, size: i32) -> PackedGaussians {
	// Safety: caller must ensure data is valid for size bytes.
	// Convert raw pointer to slice and delegate.
	unimplemented!()
}

pub fn load_spz_packed(data: &[u8]) -> PackedGaussians {
	// Parse bytes into PackedGaussians
	unimplemented!()
}

// Loads Gaussian splat from a byte pointer in packed format.
pub fn load_spz_from_ptr(data: *const u8, size: i32, options: &UnpackOptions) -> GaussianCloud {
	unimplemented!()
}

// Loads Gaussian splat data in .ply format
pub fn load_splat_from_ply(filename: &str, options: &UnpackOptions) -> GaussianCloud {
	unimplemented!()
}

pub fn compress_gzipped(data: &[u8], out: &mut Vec<u8>) -> bool {
	// Optionally use flate2 or other gzip implementation to compress data into 'out'
	unimplemented!()
}
