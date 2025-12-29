// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{path::Path, path::PathBuf};

use anyhow::{Context, Result, bail};
use arbitrary::Arbitrary;
use likely_stable::unlikely;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use crate::{
	compression, consts,
	coord::{CoordinateConverter, CoordinateSystem},
	math::{self, dim_for_degree, half_to_float},
	mmap,
	packed::PackOptions,
	packed::PackedGaussians,
	unpacked::UnpackOptions,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct GaussianSplat {
	/// The number of gaussians.
	pub num_points: i32,

	/// The degree of spherical harmonics. This must be between 0 and 3 (inclusive).
	pub spherical_harmonics_degree: i32,

	/// Whether the splat was trained with antialiasing.
	pub antialiased: bool,

	/// Positions are represented as (x, y, z) coordinates, each as a 24-bit
	/// fixed point signed integer.
	/// The number of fractional bits is determined by the `fractional_bits`
	/// field in the header.
	pub positions: Vec<f32>,

	/// Scales are represented as (x, y, z) components, each represented
	/// as an 8-bit log-encoded integer.
	pub scales: Vec<f32>,

	/// Rotations are represented as the smallest three components of the
	/// normalized rotation quaternion, for optimal rotation accuracy.
	///
	/// The largest component can be derived from the others and is not stored.
	/// Its index is stored on 2 bits and each of the smallest three
	/// components is encoded as a 10-bit signed integer.
	pub rotations: Vec<f32>,

	/// Alphas are represented as 8-bit unsigned integers.
	pub alphas: Vec<f32>,

	/// Colors are stored as (r, g, b) values, where each color component
	/// is represented as an unsigned 8-bit integer.
	pub colors: Vec<f32>,

	/// Depending on the degree of spherical harmonics for the splat,
	/// this can contain
	/// 	0 (for degree 0),
	/// 	9 (for degree 1),
	/// 	24 (for degree 2),
	/// 	45 (for degree 3)
	/// 	coefficients per gaussian.
	///
	/// The coefficients for a gaussian are organized such that the color
	/// channel is the inner (faster varying) axis, and the coefficient is
	/// the outer (slower varying) axis, i.e. for degree 1, the order of
	/// the 9 values is:
	/// 	`sh1n1_r, sh1n1_g, sh1n1_b, sh10_r, sh10_g, sh10_b, sh1p1_r, sh1p1_g, sh1p1_b`
	///
	/// Each coefficient is represented as an 8-bit signed integer.
	/// Additional quantization can be performed to attain a higher
	/// compression ratio.
	///
	/// This library currently uses 5 bits of precision for degree 0 and
	/// 4 bits of precision for degrees 1 and 2, but this may be changed
	/// in the future without breaking backwards compatibility.
	pub spherical_harmonics: Vec<f32>,
}

impl GaussianSplat {
	pub fn builder() -> GaussianSplatBuilder {
		GaussianSplatBuilder::default()
	}

	/// Loads Gaussian splat from a file in packed format, async.
	///
	/// `filepath` - gzip compressed, packed gaussian data file.
	pub async fn load_packed_from_file_into_buf_async<F>(
		filepath: F,
		unpack_opts: &UnpackOptions,
		contents: &mut Vec<u8>,
	) -> Result<Self>
	where
		F: AsRef<Path>,
	{
		let mut infile = tokio::fs::File::open(filepath).await?;

		infile.read_to_end(contents).await?;

		return Self::new_from_packed_gaussians(
			&Self::load_packed(&contents)?,
			unpack_opts,
		);
	}

	/// Loads Gaussian splat from a file in packed format, async.
	///
	/// `filepath` - gzip compressed, packed gaussian data file.
	pub async fn load_packed_from_file_async<F>(
		filepath: F,
		unpack_opts: &UnpackOptions,
	) -> Result<Self>
	where
		F: AsRef<Path>,
	{
		let mut contents = Vec::new();

		Self::load_packed_from_file_into_buf_async(filepath, unpack_opts, &mut contents)
			.await
	}

	/// Loads Gaussian splat from a file in packed format.
	///
	/// `filepath` - gzip compressed, packed gaussian data file.
	pub fn load_packed_from_file<F>(filepath: F, unpack_opts: &UnpackOptions) -> Result<Self>
	where
		F: AsRef<Path>,
	{
		// mmap on macos isn't great according to ripgrep code
		if cfg!(target_os = "macos") {
			let infile = std::fs::read(filepath)?;

			return Self::new_from_packed_gaussians(
				&Self::load_packed(&infile)?,
				unpack_opts,
			);
		}
		let mmap = mmap::open(filepath)?;
		let packed = Self::load_packed(mmap.as_ref())
			.with_context(|| "unable to load packed file")?;

		Self::new_from_packed_gaussians(&packed, unpack_opts)
	}

	/// Loads Gaussian splat from a slice of bytes in packed format.
	///
	/// `data` - gzip compressed, packed gaussian data.
	pub fn load_packed<D>(data: D) -> Result<PackedGaussians>
	where
		D: AsRef<[u8]>,
	{
		if unlikely(data.as_ref().is_empty()) {
			// we cannot return an empty struct as there is no header
			bail!("data is empty");
		}
		let mut decompressed = Vec::<u8>::new();

		crate::compression::gzip_to_bytes(data, &mut decompressed)
			.with_context(|| "unable to decompress gzip data")?;

		let packed: PackedGaussians = decompressed
			.try_into()
			.with_context(|| "unable to parse packed gaussian data")?;

		Ok(packed)
	}

	#[inline]
	pub async fn save_as_packed_async<F>(
		&self,
		filepath: F,
		pack_opts: &PackOptions,
	) -> Result<()>
	where
		F: AsRef<Path>,
	{
		let compressed = self.serialize_as_packed_bytes(pack_opts)?;

		tokio::fs::create_dir_all(
			filepath.as_ref()
				.parent()
				.ok_or_else(|| anyhow::anyhow!("recursive mkdir failed"))?,
		)
		.await?;

		tokio::fs::write(filepath, compressed)
			.await
			.with_context(|| "unable to write to file")
	}

	#[inline]
	pub fn save_as_packed<F>(&self, filepath: F, pack_opts: &PackOptions) -> Result<()>
	where
		F: AsRef<Path>,
	{
		let compressed = self.serialize_as_packed_bytes(pack_opts)?;

		std::fs::create_dir_all(
			filepath.as_ref()
				.parent()
				.ok_or_else(|| anyhow::anyhow!("recursive mkdir failed"))?,
		)?;
		std::fs::write(filepath, compressed).with_context(|| "unable to write to file")
	}

	pub fn serialize_as_packed_bytes(&self, pack_opts: &PackOptions) -> Result<Vec<u8>> {
		let packed = self.to_packed_gaussians(pack_opts)?;

		let uncompressed = packed.as_bytes_vec()?;
		let mut compressed = Vec::new();

		compression::to_gzip_bytes(uncompressed.as_ref(), &mut compressed)?;

		Ok(compressed)
	}

	pub fn new_from_packed_gaussians(
		packed: &PackedGaussians,
		unpack_opts: &UnpackOptions,
	) -> Result<Self> {
		let num_points = packed.num_points as usize;
		let sh_dim = dim_for_degree(packed.sh_degree as u8);
		let uses_float16 = packed.uses_float16();

		if unlikely(!packed.check_sizes(num_points, sh_dim, uses_float16)) {
			bail!("inconsistent sizes");
		}
		let mut result = Self {
			num_points: packed.num_points,
			spherical_harmonics_degree: packed.sh_degree,
			antialiased: packed.antialiased,
			positions: vec![0_f32; num_points * 3],
			scales: vec![0_f32; num_points * 3],
			rotations: vec![0_f32; num_points * 4],
			alphas: vec![0_f32; num_points],
			colors: vec![0_f32; num_points * 3],
			spherical_harmonics: vec![0_f32; num_points * sh_dim as usize * 3],
		};
		// positions
		if uses_float16 {
			let half_slice: &[u16] = {
				let bytes = &packed.positions;

				unsafe {
					std::slice::from_raw_parts(
						bytes.as_ptr() as *const u16,
						bytes.len() / 2,
					)
				}
			};
			for i in 0..(num_points * 3) {
				result.positions[i] = half_to_float(half_slice[i]) as f32;
			}
		} else {
			let scale = 1.0_f32 / (1_u32 << (packed.fractional_bits as u32)) as f32;

			// decode 24-bit fixed point coordinates
			for i in 0..(num_points * 3) {
				let mut fixed32 = packed.positions[i * 3 + 0] as i32;
				fixed32 |= (packed.positions[i * 3 + 1] as i32) << 8;
				fixed32 |= (packed.positions[i * 3 + 2] as i32) << 16;

				if (fixed32 & 0x800000) != 0 {
					fixed32 |= 0xff000000_u32 as i32;
				}
				result.positions[i] = fixed32 as f32 * scale;
			}
		}
		// scales
		for i in 0..(num_points * 3) {
			result.scales[i] = (packed.scales[i] as f32 / 16.0 - 10.0) as f32;
		}
		// rotations
		for i in 0..num_points {
			if packed.uses_quaternion_smallest_three {
				math::unpack_quaternion_smallest_three(
					&mut result.rotations[4 * i..4 * i + 4],
					&packed.rotations[4 * i..4 * i + 4],
				);
			} else {
				math::unpack_quaternion_first_three(
					&mut result.rotations[4 * i..4 * i + 4],
					&packed.rotations[3 * i..3 * i + 3],
				);
			}
		}
		// alphas
		for i in 0..num_points {
			result.alphas[i] = math::inv_sigmoid(packed.alphas[i] as f32 / 255.0);
		}
		// colors
		for i in 0..(num_points * 3) {
			result.colors[i] = (((packed.colors[i] as f32 / 255.0) - 0.5)
				/ consts::COLOR_SCALE) as f32;
		}
		// spherical harmonics
		for i in 0..packed.spherical_harmonics.len() {
			result.spherical_harmonics[i] =
				math::unquantize_sh(packed.spherical_harmonics[i]) as f32;
		}
		result.convert_coordinates(CoordinateSystem::RUB, unpack_opts.to_coord_sys.clone());

		Ok(result)
	}

	pub fn to_packed_gaussians(&self, pack_opts: &PackOptions) -> Result<PackedGaussians> {
		if unlikely(!self.check_sizes()) {
			bail!("inconsistent sizes");
		}
		let num_points = self.num_points as usize;
		let sh_dim = math::dim_for_degree(self.spherical_harmonics_degree as u8) as usize;
		let coord_flip = pack_opts.from.convert(CoordinateSystem::RUB);
		let fractional_bits: i32 = 12;
		let scale = (1_i32 << fractional_bits) as f32;

		let mut packed = PackedGaussians {
			num_points: self.num_points,
			sh_degree: self.spherical_harmonics_degree,
			fractional_bits,
			antialiased: self.antialiased,
			uses_quaternion_smallest_three: true,
			positions: vec![0_u8; num_points * 3 * 3],
			scales: vec![0_u8; num_points * 3],
			rotations: vec![0_u8; num_points * 4],
			alphas: vec![0_u8; num_points],
			colors: vec![0_u8; num_points * 3],
			spherical_harmonics: vec![0_u8; num_points * sh_dim * 3],
		};
		for i in 0..(num_points * 3) {
			let axis = i % 3;
			let fixed32 = (coord_flip.flip_p[axis] * self.positions[i] * scale).round()
				as i32;

			packed.positions[i * 3 + 0] = (fixed32 & 0xff) as u8;
			packed.positions[i * 3 + 1] = ((fixed32 >> 8) & 0xff) as u8;
			packed.positions[i * 3 + 2] = ((fixed32 >> 16) & 0xff) as u8;
		}
		// Pack scales
		for i in 0..(num_points * 3) {
			packed.scales[i] = math::to_u8((self.scales[i] + 10.0) * 16.0);
		}
		// Pack rotations using smallest-three encoding
		for i in 0..num_points {
			let rot_src: [f32; 4] = [
				self.rotations[4 * i],
				self.rotations[4 * i + 1],
				self.rotations[4 * i + 2],
				self.rotations[4 * i + 3],
			];
			let rot_dst = math::pack_quaternion_smallest_three(
				&rot_src,
				[
					coord_flip.flip_q[0],
					coord_flip.flip_q[1],
					coord_flip.flip_q[2],
				],
			);
			packed.rotations[4 * i..4 * i + 4].copy_from_slice(&rot_dst);
		}
		// Pack alphas with sigmoid activation
		for i in 0..num_points {
			packed.alphas[i] = math::to_u8(math::sigmoid(self.alphas[i]) * 255.0);
		}
		// Pack colors
		for i in 0..(num_points * 3) {
			packed.colors[i] = math::to_u8(
				self.colors[i] * (consts::COLOR_SCALE * 255.0) + (0.5 * 255.0),
			);
		}
		// Pack spherical harmonics
		if self.spherical_harmonics_degree > 0 {
			const SH1_BITS: i32 = 5;
			const SH_REST_BITS: i32 = 4;

			let sh_per_point = sh_dim * 3;

			for point_idx in 0..num_points {
				let base = point_idx * sh_per_point;

				let mut j = 0_usize;
				let mut k = 0_usize;

				while j < 9 && j < sh_per_point {
					let step = 1_i32 << (8 - SH1_BITS);

					packed.spherical_harmonics[base + j + 0] =
						math::quantize_sh(
							coord_flip.flip_sh[k]
								* self.spherical_harmonics
									[base + j + 0],
							step,
						);
					packed.spherical_harmonics[base + j + 1] =
						math::quantize_sh(
							coord_flip.flip_sh[k]
								* self.spherical_harmonics
									[base + j + 1],
							step,
						);
					packed.spherical_harmonics[base + j + 2] =
						math::quantize_sh(
							coord_flip.flip_sh[k]
								* self.spherical_harmonics
									[base + j + 2],
							step,
						);
					j += 3;
					k += 1;
				}
				while j < sh_per_point {
					let step = 1i32 << (8 - SH_REST_BITS);

					packed.spherical_harmonics[base + j + 0] =
						math::quantize_sh(
							coord_flip.flip_sh[k]
								* self.spherical_harmonics
									[base + j + 0],
							step,
						);
					packed.spherical_harmonics[base + j + 1] =
						math::quantize_sh(
							coord_flip.flip_sh[k]
								* self.spherical_harmonics
									[base + j + 1],
							step,
						);
					packed.spherical_harmonics[base + j + 2] =
						math::quantize_sh(
							coord_flip.flip_sh[k]
								* self.spherical_harmonics
									[base + j + 2],
							step,
						);
					j += 3;
					k += 1;
				}
			}
		}
		Ok(packed)
	}

	pub fn convert_coordinates(
		&mut self,
		from: crate::coord::CoordinateSystem,
		to: crate::coord::CoordinateSystem,
	) {
		if unlikely(self.num_points == 0) {
			return;
		}
		let (x_match, y_match, z_match) = from.axes_match(to);

		let x = if x_match { 1.0_f32 } else { -1.0_f32 };
		let y = if y_match { 1.0_f32 } else { -1.0_f32 };
		let z = if z_match { 1.0_f32 } else { -1.0_f32 };

		let flip = CoordinateConverter {
			flip_p: [x, y, z],
			flip_q: [y * z, x * z, x * y],
			flip_sh: [
				y,         // 0
				z,         // 1
				x,         // 2
				x * y,     // 3
				y * z,     // 4
				1.0_f32,   // 5
				x * z,     // 6
				1.0_f32,   // 7
				y,         // 8
				x * y * z, // 9
				y,         // 10
				z,         // 11
				x,         // 12
				z,         // 13
				x,         // 14
			],
		};
		for i in (0..self.positions.len()).step_by(3) {
			self.positions[i + 0] *= flip.flip_p[0];
			self.positions[i + 1] *= flip.flip_p[1];
			self.positions[i + 2] *= flip.flip_p[2];
		}
		for i in (0..self.rotations.len()).step_by(4) {
			self.rotations[i + 0] *= flip.flip_q[0];
			self.rotations[i + 1] *= flip.flip_q[1];
			self.rotations[i + 2] *= flip.flip_q[2];
			// rotations[i + 3] (w) unchanged
		}
		let total_coeffs = if self.spherical_harmonics.len() >= 3 {
			self.spherical_harmonics.len() / 3
		} else {
			0
		};
		let num_points = self.num_points.max(0) as usize;

		if unlikely(num_points == 0 || total_coeffs == 0) {
			return;
		}
		let coeffs_per_point = total_coeffs / num_points;
		let mut idx = 0_usize;

		for _pt in 0..num_points {
			for j in 0..coeffs_per_point {
				let f = flip.flip_sh[j];

				self.spherical_harmonics[idx + 0] *= f;
				self.spherical_harmonics[idx + 1] *= f;
				self.spherical_harmonics[idx + 2] *= f;

				idx += 3;
			}
		}
	}

	/// Rotate 180 degrees about X axis (RUB <-> RDF).
	#[inline]
	pub fn rotate_180_deg_about_x(&mut self) {
		self.convert_coordinates(
			crate::coord::CoordinateSystem::RUB,
			crate::coord::CoordinateSystem::RDF,
		);
	}

	/// Compute median ellipsoid volume.
	pub fn median_volume(&self) -> f32 {
		if unlikely(self.scales.is_empty()) {
			return 0.01;
		}
		// The volume of an ellipsoid is 4/3 * pi * x * y * z,
		// where x, y, and z are the radii on each axis.
		// Scales are stored on a log scale, and
		// 	exp(x) * exp(y) * exp(z) = exp(x + y + z).
		// So we can sort by value = (x + y + z) and compute
		// 	volume = 4/3 * pi * exp(value) later.
		let mut sums = self
			.scales
			.chunks_exact(3)
			.filter_map(|c| {
				let s = c[0] + c[1] + c[2];

				if unlikely(!s.is_finite()) {
					None
				} else {
					Some(s)
				}
			})
			.collect::<Vec<_>>();

		if unlikely(sums.is_empty()) {
			return 0.01;
		}
		let n = sums.len() / 2;

		sums.select_nth_unstable_by(n, |a, b| {
			a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
		});
		let median = sums[sums.len() / 2];

		if unlikely(!median.is_finite() || median <= f32::MIN_POSITIVE.ln()) {
			return 0.01;
		}
		(std::f32::consts::PI * 4.0 / 3.0) * median.exp()
	}

	pub fn check_sizes(&self) -> bool {
		if self.num_points < 0 {
			return false;
		}
		if self.spherical_harmonics_degree < 0 || self.spherical_harmonics_degree > 3 {
			return false;
		}
		let np = self.num_points as usize;
		let sh_dim = dim_for_degree(self.spherical_harmonics_degree as u8) as usize;

		let expected_xyz = np.saturating_mul(3);
		let expected_rot = np.saturating_mul(4);
		let expected_colors = np.saturating_mul(3);
		let expected_sh = np.saturating_mul(sh_dim).saturating_mul(3);

		if self.positions.len() != expected_xyz
			|| self.scales.len() != expected_xyz
			|| self.rotations.len() != expected_rot
			|| self.alphas.len() != np
			|| self.colors.len() != expected_colors
			|| self.spherical_harmonics.len() != expected_sh
		{
			return false;
		}
		true
	}

	pub fn bbox(&self) -> BoundingBox {
		let mut min_x = self.positions[0];
		let mut max_x = self.positions[0];
		let mut min_y = self.positions[1];
		let mut max_y = self.positions[1];
		let mut min_z = self.positions[2];
		let mut max_z = self.positions[2];

		for i in (0..self.positions.len()).step_by(3) {
			min_x = min_x.min(self.positions[i]);
			max_x = max_x.max(self.positions[i]);
			min_y = min_y.min(self.positions[i + 1]);
			max_y = max_y.max(self.positions[i + 1]);
			min_z = min_z.min(self.positions[i + 2]);
			max_z = max_z.max(self.positions[i + 2]);
		}
		BoundingBox {
			min_x,
			max_x,
			min_y,
			max_y,
			min_z,
			max_z,
		}
	}
}

impl std::fmt::Display for GaussianSplat {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let _ = write!(
			f,
			"GaussianSplat={{num_points={}, sh_degree={}, antialiased={}, median_ellipsoid_volume={}, ",
			self.num_points,
			self.spherical_harmonics_degree,
			self.antialiased,
			self.median_volume()
		);
		let BoundingBox {
			min_x,
			max_x,
			min_y,
			max_y,
			min_z,
			max_z,
		} = self.bbox();

		write!(
			f,
			"bbox=[x={:.6} to {:.6}, y={:.6} to {:.6}, z={:.6} to {:.6}]}}",
			min_x, max_x, min_y, max_y, min_z, max_z
		)?;
		Ok(())
	}
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct BoundingBox {
	pub min_x: f32,
	pub max_x: f32,
	pub min_y: f32,
	pub max_y: f32,
	pub min_z: f32,
	pub max_z: f32,
}

impl BoundingBox {
	pub fn size(&self) -> (f32, f32, f32) {
		(
			self.max_x - self.min_x,
			self.max_y - self.min_y,
			self.max_z - self.min_z,
		)
	}

	/// Get the center of the bounding box.
	///
	/// Returns:
	/// 	A tuple of (x, y, z) center coordinates.
	pub fn center(&self) -> (f32, f32, f32) {
		(
			(self.min_x + self.max_x) / 2.0,
			(self.min_y + self.max_y) / 2.0,
			(self.min_z + self.max_z) / 2.0,
		)
	}
}

#[derive(Clone, Debug, Arbitrary)]
pub struct GaussianSplatBuilder {
	filepath: Option<PathBuf>,
	unpack_opts: UnpackOptions,
	packed: bool,
}

impl Default for GaussianSplatBuilder {
	#[inline]
	fn default() -> Self {
		GaussianSplatBuilder {
			filepath: None,
			unpack_opts: UnpackOptions::default(),
			packed: true,
		}
	}
}

impl GaussianSplatBuilder {
	pub fn filepath<F>(mut self, filepath: F) -> Self
	where
		F: AsRef<Path>,
	{
		self.filepath = Some(filepath.as_ref().to_path_buf());
		self
	}

	pub fn packed(mut self, packed: bool) -> Result<Self> {
		if !packed {
			bail!("only packed format loading is supported currently");
		}
		self.packed = packed;

		Ok(self)
	}

	pub fn unpack_options(mut self, opts: UnpackOptions) -> Self {
		self.unpack_opts = opts;
		self
	}

	pub fn load(self) -> Result<GaussianSplat> {
		GaussianSplat::load_packed_from_file(
			self.filepath.as_ref().unwrap(),
			&self.unpack_opts,
		)
	}

	pub async fn load_async(self) -> Result<GaussianSplat> {
		GaussianSplat::load_packed_from_file_async(
			self.filepath.as_ref().unwrap(),
			&self.unpack_opts,
		)
		.await
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use approx::assert_relative_eq;
	use rstest::rstest;

	#[rstest]
	#[case(
		GaussianSplat::default(),
		vec![
			-1.0, -1.0, -1.0, // First gaussian: scale sum = -3
			0.0, 0.0, 0.0, // Second gaussian: scale sum = 0
			1.0, 1.0, 1.0, // Third gaussian: scale sum = 3
		],
		(4.0 / 3.0) * std::f32::consts::PI * 0.0_f32.exp(),
		1e-5_f32,
	)]
	#[case(
		GaussianSplat::default(),
		vec![
			-2.0, -2.0, -2.0, // First gaussian: scale sum = -6
			-1.0, -1.0, -1.0, // Second gaussian: scale sum = -3
			0.0, 0.0, 0.0, // Third gaussian: scale sum = 0 (median)
			1.0, 1.0, 1.0, // Fourth gaussian: scale sum = 3
			2.0, 2.0, 2.0, // Fifth gaussian: scale sum = 6
		],
		(4.0 / 3.0) * std::f32::consts::PI * 0.0_f32.exp(),
		1e-5_f32,
	)]
	// Set up scales for volume calculation (log scale)
	// volume = 4/3 * pi * exp(scale_sum)
	// use known values: scales [-1,-1,-1], [0,0,0], [1,1,1]
	// scale sums: -3, 0, 3
	// median scale sum: 0
	// expected median volume: 4/3 * pi * exp(0) = 4/3 * pi
	fn test_median_volume(
		#[case] mut gs: GaussianSplat,
		#[case] scales: Vec<f32>,
		#[case] expected_vol: f32,
		#[case] epsilon: f32,
	) {
		gs.scales = scales;

		assert_relative_eq!(gs.median_volume(), expected_vol, epsilon = epsilon);
	}
}
