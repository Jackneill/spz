// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::Path;
use std::{fmt::Write, io::Read};

use anyhow::{Context, Result, bail};
use arbitrary::Arbitrary;
use likely_stable::unlikely;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use crate::{
	compression, consts,
	coord::{AxisFlips, CoordinateSystem},
	header::{Flags, Header},
	math::{self, dim_for_degree},
	mmap,
	packed::PackedGaussianSplat,
};

/// A set of Gaussian Splats representing a 3D scene.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Arbitrary)]
pub struct GaussianSplat {
	/// Header data for the splat.
	pub header: Header,

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
	#[inline]
	pub fn builder() -> GaussianSplatBuilder {
		GaussianSplatBuilder::default()
	}

	/// Loads a [`GaussianSplat`] from a file with the given options, async.
	///
	/// `filepath` - gzip compressed, packed gaussian data file.
	/// `opts` - options for loading the splat.
	#[inline]
	pub async fn load_with_into_buf_async<F>(
		filepath: F,
		opts: &LoadOptions,
		contents: &mut Vec<u8>,
	) -> Result<Self>
	where
		F: AsRef<Path>,
	{
		let mut infile = tokio::fs::File::open(filepath).await?;

		infile.read_to_end(contents).await?;

		return Self::new_from_packed_gaussians(
			&PackedGaussianSplat::from_bytes(&contents)?,
			opts,
		);
	}

	/// Loads a [`GaussianSplat`] from a file with the given options from
	/// the reader, async.
	///
	/// `from` - gzip compressed, packed gaussian data.
	/// `opts` - options for loading the splat.
	#[inline]
	pub async fn read_from_async<F, R>(mut from: R, opts: &LoadOptions) -> Result<Self>
	where
		F: AsRef<Path>,
		R: AsyncReadExt + Unpin,
	{
		let mut contents = Vec::new();

		from.read_to_end(&mut contents).await?;

		let packed = PackedGaussianSplat::from_bytes(&contents)
			.with_context(|| "unable to parse splat")?;

		Self::new_from_packed_gaussians(&packed, opts)
	}

	/// Loads a [`GaussianSplat`] from a file with the given options from
	/// the reader.
	///
	/// `from` - gzip compressed, packed gaussian data.
	/// `opts` - options for loading the splat.
	#[inline]
	pub fn read_from<F, R>(mut from: R, opts: &LoadOptions) -> Result<Self>
	where
		F: AsRef<Path>,
		R: Read,
	{
		let mut contents = Vec::new();

		from.read_to_end(&mut contents)?;

		let packed = PackedGaussianSplat::from_bytes(&contents)
			.with_context(|| "unable to parse splat")?;

		Self::new_from_packed_gaussians(&packed, opts)
	}

	/// Loads a [`GaussianSplat`] from a file with the given options, async.
	///
	/// `filepath` - gzip compressed, packed gaussian data file.
	/// `opts` - options for loading the splat.
	#[inline]
	pub async fn load_with_async<F>(filepath: F, opts: &LoadOptions) -> Result<Self>
	where
		F: AsRef<Path>,
	{
		let mut contents = Vec::new();

		Self::load_with_into_buf_async(filepath, opts, &mut contents).await
	}

	/// Loads a [`GaussianSplat`] from a file with the given options.
	///
	/// `filepath` - gzip compressed, packed gaussian data file.
	/// `opts` - options for loading the splat.
	#[inline]
	pub fn load_with<F>(filepath: F, opts: &LoadOptions) -> Result<Self>
	where
		F: AsRef<Path>,
	{
		// mmap on macos isn't great according to ripgrep code
		if cfg!(target_os = "macos") {
			let infile = std::fs::read(filepath)?;

			return Self::new_from_packed_gaussians(
				&PackedGaussianSplat::from_bytes(&infile)?,
				opts,
			);
		}
		let mmap = mmap::mmap(filepath)?;
		let packed = PackedGaussianSplat::from_bytes(mmap.as_ref())
			.with_context(|| "unable to load packed file")?;

		Self::new_from_packed_gaussians(&packed, opts)
	}

	/// Loads a [`GaussianSplat`] from a file.
	///
	/// Convenience method that uses the default load options.
	///
	/// `filepath` - gzip compressed, packed gaussian data file.
	#[inline]
	pub fn load<F>(filepath: F) -> Result<Self>
	where
		F: AsRef<Path>,
	{
		Self::load_with(
			filepath,
			&LoadOptions::builder()
				.coord_sys(CoordinateSystem::default())
				.build(),
		)
	}

	/// Loads a [`GaussianSplat`] from a file, async.
	///
	/// Convenience async method that uses default load options.
	///
	/// `filepath` - gzip compressed, packed gaussian data file.
	#[inline]
	pub async fn load_async<F>(filepath: F) -> Result<Self>
	where
		F: AsRef<Path>,
	{
		Self::load_with_async(
			filepath,
			&LoadOptions::builder()
				.coord_sys(CoordinateSystem::default()) // default to LUF (glTF)
				.build(),
		)
		.await
	}

	/// Saves a [`GaussianSplat`] to a file.
	///
	/// `filepath` - file path to save the gzip compressed, packed gaussian data.
	/// `opts` - options for saving the splat.
	#[inline]
	pub async fn save_async<F>(&self, filepath: F, opts: &SaveOptions) -> Result<()>
	where
		F: AsRef<Path>,
	{
		let compressed = self.serialize_to_packed_bytes(opts)?;

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

	/// Saves a [`GaussianSplat`] to a file.
	///
	/// `filepath` - file path to save the gzip compressed, packed gaussian data.
	/// `opts` - options for saving the splat.
	#[inline]
	pub fn save<F>(&self, filepath: F, opts: &SaveOptions) -> Result<()>
	where
		F: AsRef<Path>,
	{
		let compressed = self.serialize_to_packed_bytes(opts)?;

		std::fs::create_dir_all(
			filepath.as_ref()
				.parent()
				.ok_or_else(|| anyhow::anyhow!("recursive mkdir failed"))?,
		)?;
		std::fs::write(filepath, compressed).with_context(|| "unable to write to file")
	}

	pub fn serialize_to_packed_bytes(&self, opts: &SaveOptions) -> Result<Vec<u8>> {
		let packed = self.to_packed_gaussians(opts)?;

		let uncompressed = packed.to_bytes_vec()?;
		let mut compressed = Vec::new();

		compression::gzip::compress_bytes(uncompressed.as_ref(), &mut compressed)?;

		Ok(compressed)
	}

	pub fn new_from_packed_gaussians(
		packed: &PackedGaussianSplat,
		opts: &LoadOptions,
	) -> Result<Self> {
		let num_points = packed.num_points as usize;
		let sh_dim = dim_for_degree(packed.sh_degree as u8);

		if unlikely(!packed.check_sizes(num_points, sh_dim)) {
			bail!("inconsistent sizes");
		}
		let mut result = Self {
			header: Header {
				num_points: packed.num_points,
				spherical_harmonics_degree: packed.sh_degree as u8,
				fractional_bits: packed.fractional_bits as u8,
				flags: if packed.antialiased {
					Flags::ANTIALIASED
				} else {
					Flags::none()
				},
				..Default::default()
			},
			positions: vec![0_f32; num_points * 3],
			scales: vec![0_f32; num_points * 3],
			rotations: vec![0_f32; num_points * 4],
			alphas: vec![0_f32; num_points],
			colors: vec![0_f32; num_points * 3],
			spherical_harmonics: vec![0_f32; num_points * sh_dim as usize * 3],
		};
		// positions: decode 24-bit fixed point coordinates
		let scale = 1.0_f32 / (1_u32 << (packed.fractional_bits as u32)) as f32;

		for (dst, src) in result
			.positions
			.chunks_exact_mut(3)
			.zip(packed.positions.chunks_exact(9))
		{
			for i in 0..3 {
				let base = i * 3;
				let mut fixed32 = src[base] as i32
					| ((src[base + 1] as i32) << 8) | ((src[base + 2]
					as i32) << 16);

				if (fixed32 & 0x800000) != 0 {
					fixed32 |= 0xff000000_u32 as i32;
				}
				dst[i] = fixed32 as f32 * scale;
			}
		}
		// scales
		for (dst, src) in result.scales.iter_mut().zip(packed.scales.iter()) {
			*dst = (*src as f32 / 16.0 - 10.0) as f32;
		}
		// rotations
		if packed.uses_quaternion_smallest_three {
			for (dst, src) in result
				.rotations
				.chunks_exact_mut(4)
				.zip(packed.rotations.chunks_exact(4))
			{
				math::unpack_quaternion_smallest_three(dst, src);
			}
		} else {
			for (dst, src) in result
				.rotations
				.chunks_exact_mut(4)
				.zip(packed.rotations.chunks_exact(3))
			{
				math::unpack_quaternion_first_three(dst, src);
			}
		}
		// alphas
		for (dst, src) in result.alphas.iter_mut().zip(packed.alphas.iter()) {
			*dst = math::inv_sigmoid(*src as f32 / 255.0);
		}
		// colors
		for (dst, src) in result.colors.iter_mut().zip(packed.colors.iter()) {
			*dst = ((*src as f32 / 255.0) - 0.5) / consts::COLOR_SCALE;
		}
		// spherical harmonics
		for (dst, src) in result
			.spherical_harmonics
			.iter_mut()
			.zip(packed.spherical_harmonics.iter())
		{
			*dst = math::unquantize_sh(*src);
		}
		result.convert_coordinates(opts.coord_sys.clone(), CoordinateSystem::RightUpBack);

		Ok(result)
	}

	pub fn to_packed_gaussians(&self, opts: &SaveOptions) -> Result<PackedGaussianSplat> {
		if unlikely(!self.check_sizes()) {
			bail!("inconsistent sizes");
		}
		let num_points = self.header.num_points as usize;
		let sh_dim =
			math::dim_for_degree(self.header.spherical_harmonics_degree as u8) as usize;
		let axis_flips = opts.coord_sys.axis_flips_to(CoordinateSystem::RightUpBack);
		// Use 12 bits for the fractional part of coordinates (~0.25mm resolution).
		let fractional_bits: i32 = 12;
		let scale = (1_i32 << fractional_bits) as f32;

		let mut packed = PackedGaussianSplat {
			num_points: self.header.num_points,
			sh_degree: self.header.spherical_harmonics_degree as i32,
			fractional_bits,
			antialiased: self.header.flags.is_antialiased(),
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
			let fixed32 = (axis_flips.position[axis] * self.positions[i] * scale)
				.round() as i32;

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
					axis_flips.rotation[0],
					axis_flips.rotation[1],
					axis_flips.rotation[2],
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
		if self.header.spherical_harmonics_degree > 0 {
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
							axis_flips.spherical_harmonics[k]
								* self.spherical_harmonics
									[base + j + 0],
							step,
						);
					packed.spherical_harmonics[base + j + 1] =
						math::quantize_sh(
							axis_flips.spherical_harmonics[k]
								* self.spherical_harmonics
									[base + j + 1],
							step,
						);
					packed.spherical_harmonics[base + j + 2] =
						math::quantize_sh(
							axis_flips.spherical_harmonics[k]
								* self.spherical_harmonics
									[base + j + 2],
							step,
						);
					j += 3;
					k += 1;
				}
				while j < sh_per_point {
					let step = 1_i32 << (8 - SH_REST_BITS);

					packed.spherical_harmonics[base + j + 0] =
						math::quantize_sh(
							axis_flips.spherical_harmonics[k]
								* self.spherical_harmonics
									[base + j + 0],
							step,
						);
					packed.spherical_harmonics[base + j + 1] =
						math::quantize_sh(
							axis_flips.spherical_harmonics[k]
								* self.spherical_harmonics
									[base + j + 1],
							step,
						);
					packed.spherical_harmonics[base + j + 2] =
						math::quantize_sh(
							axis_flips.spherical_harmonics[k]
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
		source_cs: crate::coord::CoordinateSystem,
		target_cs: crate::coord::CoordinateSystem,
	) {
		if unlikely(self.header.num_points == 0) {
			return;
		}
		let (x_match, y_match, z_match) = source_cs.axes_align(target_cs);

		let x = if x_match { 1.0_f32 } else { -1.0_f32 };
		let y = if y_match { 1.0_f32 } else { -1.0_f32 };
		let z = if z_match { 1.0_f32 } else { -1.0_f32 };

		let flip = AxisFlips {
			position: [x, y, z],
			rotation: [y * z, x * z, x * y],
			spherical_harmonics: [
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
			self.positions[i + 0] *= flip.position[0];
			self.positions[i + 1] *= flip.position[1];
			self.positions[i + 2] *= flip.position[2];
		}
		for i in (0..self.rotations.len()).step_by(4) {
			self.rotations[i + 0] *= flip.rotation[0];
			self.rotations[i + 1] *= flip.rotation[1];
			self.rotations[i + 2] *= flip.rotation[2];
			// rotations[i + 3] (w) unchanged
		}
		let total_coeffs = if self.spherical_harmonics.len() >= 3 {
			self.spherical_harmonics.len() / 3
		} else {
			0
		};
		let num_points = self.header.num_points.max(0) as usize;

		if unlikely(num_points == 0 || total_coeffs == 0) {
			return;
		}
		let coeffs_per_point = total_coeffs / num_points;
		let mut idx = 0_usize;

		for _pt in 0..num_points {
			for j in 0..coeffs_per_point {
				let f = flip.spherical_harmonics[j];

				self.spherical_harmonics[idx + 0] *= f;
				self.spherical_harmonics[idx + 1] *= f;
				self.spherical_harmonics[idx + 2] *= f;

				idx += 3;
			}
		}
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

	/// Validates that all internal arrays have consistent sizes.
	///
	/// Checks that:
	/// - `num_points` is non-negative
	/// - `spherical_harmonics_degree` is in range 0..=3
	/// - `positions` has length `num_points * 3`
	/// - `scales` has length `num_points * 3`
	/// - `rotations` has length `num_points * 4`
	/// - `alphas` has length `num_points`
	/// - `colors` has length `num_points * 3`
	/// - `spherical_harmonics` has length `num_points * sh_dim * 3`
	///
	/// # Returns
	///
	/// Returns `true` if all sizes are valid, `false` otherwise.
	pub fn check_sizes(&self) -> bool {
		if unlikely(self.header.num_points < 0) {
			return false;
		}
		if unlikely(self.header.spherical_harmonics_degree > 3) {
			return false;
		}
		let np = self.header.num_points as usize;
		let sh_dim = dim_for_degree(self.header.spherical_harmonics_degree as u8) as usize;

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

	pub fn pretty_fmt(&self) -> String {
		let bbox = self.bbox();
		let (size_x, size_y, size_z) = bbox.size();
		let (center_x, center_y, center_z) = bbox.center();

		let mut ret = String::new();

		let _ = write!(ret, "GaussianSplat:\n");
		let _ = write!(ret, "\tNumber of points:\t\t{}\n", self.header.num_points);
		let _ = write!(
			ret,
			"\tSpherical harmonics degree:\t{}\n",
			self.header.spherical_harmonics_degree
		);
		let _ = write!(
			ret,
			"\tAntialiased:\t\t\t{}\n",
			self.header.flags.is_antialiased()
		);
		let _ = write!(
			ret,
			"\tMedian ellipsoid volume:\t{:}\n",
			self.median_volume()
		);
		let _ = write!(
			ret,
			"\tBounding box:\n\t\tx: {:} to {:} (size {:}, center {:})\n",
			bbox.min_x, bbox.max_x, size_x, center_x
		);
		let _ = write!(
			ret,
			"\t\ty: {:} to {:} (size {:}, center {:})\n",
			bbox.min_y, bbox.max_y, size_y, center_y
		);
		let _ = write!(
			ret,
			"\t\tz: {:} to {:} (size {:}, center {:})\n",
			bbox.min_z, bbox.max_z, size_z, center_z
		);
		ret
	}
}

impl std::fmt::Display for GaussianSplat {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let _ = write!(
			f,
			"GaussianSplat={{{}, {}, ",
			self.header,
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

#[derive(Clone, Debug, Arbitrary)]
pub struct GaussianSplatBuilder {
	coord_sys: CoordinateSystem,
	packed: bool,
}

impl GaussianSplatBuilder {
	pub fn packed(mut self, packed: bool) -> Result<Self> {
		if unlikely(!packed) {
			bail!("only packed format loading is supported currently");
		}
		self.packed = packed;

		Ok(self)
	}

	pub fn coord_sys(mut self, coord_sys: CoordinateSystem) -> Self {
		self.coord_sys = coord_sys;
		self
	}

	pub fn load<P>(self, filepath: P) -> Result<GaussianSplat>
	where
		P: AsRef<Path>,
	{
		GaussianSplat::load_with(
			filepath,
			&LoadOptions::builder().coord_sys(self.coord_sys).build(),
		)
	}

	pub async fn load_async<P>(self, filepath: P) -> Result<GaussianSplat>
	where
		P: AsRef<Path>,
	{
		GaussianSplat::load_with_async(
			filepath,
			&LoadOptions::builder().coord_sys(self.coord_sys).build(),
		)
		.await
	}
}

impl Default for GaussianSplatBuilder {
	#[inline]
	fn default() -> Self {
		GaussianSplatBuilder {
			coord_sys: CoordinateSystem::Unspecified,
			packed: true,
		}
	}
}

#[derive(Clone, Debug, Arbitrary)]
pub struct LoadOptionsBuilder {
	coord_sys: CoordinateSystem,
}

impl LoadOptionsBuilder {
	#[inline]
	pub fn coord_sys(mut self, coord_sys: CoordinateSystem) -> Self {
		self.coord_sys = coord_sys;
		self
	}

	#[inline]
	pub fn build(self) -> LoadOptions {
		LoadOptions {
			coord_sys: self.coord_sys,
		}
	}
}

impl Default for LoadOptionsBuilder {
	#[inline]
	fn default() -> Self {
		Self {
			coord_sys: CoordinateSystem::Unspecified,
		}
	}
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Arbitrary)]
pub struct LoadOptions {
	/// Specifies the coordinate system to convert to when loading from
	/// the one the data is stored in the SPZ file.
	///
	/// For more information see [`CoordinateSystem`](crate::coord::CoordinateSystem).
	pub coord_sys: CoordinateSystem,
}

impl LoadOptions {
	#[inline]
	pub fn builder() -> LoadOptionsBuilder {
		LoadOptionsBuilder::default()
	}
}

/// Options for saving the [`GaussianSplat`](crate::gaussian_splat::GaussianSplat) data.
///
/// Specifies the source coordinate system so axis flips can be applied during
/// compression to convert to the SPZ internal format (RightUpBack|RUB).
///
/// For more information see [`CoordinateSystem`](crate::coord::CoordinateSystem).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Arbitrary)]
pub struct SaveOptions {
	/// Specifies the coordinate system to convert to when saving the
	/// Gaussian Splat data into the SPZ file.
	pub coord_sys: CoordinateSystem,
}

impl SaveOptions {
	/// Creates a new [`SaveOptionsBuilder`].
	#[inline]
	pub fn builder() -> SaveOptionsBuilder {
		SaveOptionsBuilder::default()
	}
}

/// Builder for [`SaveOptions`].
#[derive(Clone, Debug, Arbitrary)]
pub struct SaveOptionsBuilder {
	coord_sys: CoordinateSystem,
}

impl SaveOptionsBuilder {
	/// Sets the source coordinate system.
	#[inline]
	pub fn coord_sys(mut self, coord_sys: CoordinateSystem) -> Self {
		self.coord_sys = coord_sys;
		self
	}

	/// Builds the [`SaveOptions`].
	#[inline]
	pub fn build(self) -> SaveOptions {
		SaveOptions {
			coord_sys: self.coord_sys,
		}
	}
}

impl Default for SaveOptionsBuilder {
	#[inline]
	fn default() -> Self {
		Self {
			coord_sys: CoordinateSystem::Unspecified,
		}
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
	fn test_median_volume(
		#[case] mut gs: GaussianSplat,
		#[case] scales: Vec<f32>,
		#[case] expected_vol: f32,
		#[case] epsilon: f32,
	) {
		gs.scales = scales;

		assert_relative_eq!(gs.median_volume(), expected_vol, epsilon = epsilon);
	}

	#[rstest]
	#[case(vec![])]
	#[case(vec![f32::NAN, f32::NAN, f32::NAN])]
	#[case(vec![f32::INFINITY, f32::INFINITY, f32::INFINITY])]
	fn test_median_volume_fallback(#[case] scales: Vec<f32>) {
		let mut gs = GaussianSplat::default();
		gs.scales = scales;

		assert_relative_eq!(gs.median_volume(), 0.01, epsilon = 1e-6);
	}

	#[test]
	fn test_check_sizes_default_is_valid() {
		let gs = GaussianSplat::default();

		assert!(gs.check_sizes());
	}

	#[test]
	fn test_check_sizes_negative_num_points() {
		let mut gs = GaussianSplat::default();

		gs.header.num_points = -1;

		assert!(!gs.check_sizes());
	}

	#[test]
	fn test_check_sizes_invalid_sh_degree() {
		let mut gs = GaussianSplat::default();

		gs.header.spherical_harmonics_degree = 4;

		assert!(!gs.check_sizes());
	}

	#[test]
	fn test_check_sizes_wrong_positions_len() {
		let gs = GaussianSplat {
			header: Header {
				num_points: 1,
				..Default::default()
			},
			positions: vec![0.0; 2],
			scales: vec![0.0; 3],
			rotations: vec![0.0; 4],
			alphas: vec![0.0; 1],
			colors: vec![0.0; 3],
			spherical_harmonics: vec![],
		};
		assert!(!gs.check_sizes());
	}

	#[test]
	fn test_check_sizes_correct_for_one_point_sh1() {
		let gs = GaussianSplat {
			header: Header {
				num_points: 1,
				spherical_harmonics_degree: 1,
				..Default::default()
			},
			positions: vec![0.0; 3],
			scales: vec![0.0; 3],
			rotations: vec![0.0; 4],
			alphas: vec![0.0; 1],
			colors: vec![0.0; 3],
			spherical_harmonics: vec![0.0; 9],
		};
		assert!(gs.check_sizes());
	}

	#[rstest]
	#[case((-1.0, 1.0, 0.0, 5.0, -3.0, 3.0), (2.0, 5.0, 6.0), (0.0, 2.5, 0.0))]
	#[case((-2.0, 2.0, -1.0, 3.0, 0.0, 10.0), (4.0, 4.0, 10.0), (0.0, 1.0, 5.0))]
	#[case((5.0, 5.0, 5.0, 5.0, 5.0, 5.0), (0.0, 0.0, 0.0), (5.0, 5.0, 5.0))]
	fn test_bounding_box(
		#[case] bounds: (f32, f32, f32, f32, f32, f32),
		#[case] expected_size: (f32, f32, f32),
		#[case] expected_center: (f32, f32, f32),
	) {
		let bbox = BoundingBox {
			min_x: bounds.0,
			max_x: bounds.1,
			min_y: bounds.2,
			max_y: bounds.3,
			min_z: bounds.4,
			max_z: bounds.5,
		};
		assert_eq!(bbox.size(), expected_size);
		assert_eq!(bbox.center(), expected_center);
	}

	#[test]
	fn test_builder_default() {
		let builder = GaussianSplatBuilder::default();

		assert_eq!(builder.coord_sys, CoordinateSystem::Unspecified);
	}

	#[rstest]
	#[case(false, true)]
	#[case(true, false)]
	fn test_builder_packed(#[case] packed: bool, #[case] expect_err: bool) {
		let builder = GaussianSplatBuilder::default();

		assert_eq!(builder.packed(packed).is_err(), expect_err);
	}

	#[test]
	fn test_builder_coord_sys() {
		let builder = GaussianSplat::builder().coord_sys(CoordinateSystem::RightDownFront);

		assert_eq!(builder.coord_sys, CoordinateSystem::RightDownFront);
	}

	#[test]
	fn test_load_options_builder() {
		let opts = LoadOptions::builder()
			.coord_sys(CoordinateSystem::LeftUpFront)
			.build();

		assert_eq!(opts.coord_sys, CoordinateSystem::LeftUpFront);
	}

	#[test]
	fn test_save_options_builder() {
		let opts = SaveOptions::builder()
			.coord_sys(CoordinateSystem::RightUpFront)
			.build();

		assert_eq!(opts.coord_sys, CoordinateSystem::RightUpFront);
	}

	#[test]
	fn test_convert_coordinates_zero_points_noop() {
		let mut gs = GaussianSplat::default();

		gs.convert_coordinates(
			CoordinateSystem::RightUpBack,
			CoordinateSystem::RightDownFront,
		);
		assert!(gs.positions.is_empty());
	}

	#[test]
	fn test_convert_coordinates_unspecified_noop() {
		let mut gs = GaussianSplat {
			header: Header {
				num_points: 1,
				..Default::default()
			},
			positions: vec![1.0, 2.0, 3.0],
			scales: vec![0.0; 3],
			rotations: vec![0.0, 0.0, 0.0, 1.0],
			alphas: vec![0.5],
			colors: vec![0.0; 3],
			spherical_harmonics: vec![],
		};
		let original_pos = gs.positions.clone();

		gs.convert_coordinates(
			CoordinateSystem::Unspecified,
			CoordinateSystem::RightDownFront,
		);
		assert_eq!(gs.positions, original_pos);
	}

	#[test]
	fn test_to_packed_gaussians_inconsistent_sizes_fails() {
		let gs = GaussianSplat {
			header: Header {
				num_points: 1,
				..Default::default()
			},
			positions: vec![1.0],
			..Default::default()
		};
		assert!(gs.to_packed_gaussians(&SaveOptions::default()).is_err());
	}
}
