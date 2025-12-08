use std::{fs::File, path::Path};

use anyhow::Result;

use crate::{
	gaussians::PackedGaussians,
	types::{CoordFlip, PackOptions},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GaussianCloud {
	pub num_points: usize,
	pub sh_degree: usize,
	pub antialiased: bool,
	pub positions: Vec<f64>,
	pub scales: Vec<f64>,
	pub rotations: Vec<f64>,
	pub alphas: Vec<f64>,
	pub colors: Vec<f64>,
	pub sh: Vec<f64>,
}

impl GaussianCloud {
	/// Loads Gaussian splat from a file in packed format.
	///
	/// `filepath` is gzip compressed, packed gaussian data file.
	pub fn load_packed_from_file<F>(filepath: F) -> Result<GaussianCloud>
	where
		F: AsRef<Path>,
	{
		// mmap on macos isn't great according to ripgrep code
		if cfg!(target_os = "macos") {
			let infile = std::fs::read(filepath)?;

			return Self::load_packed(&infile);
		}
		let file = File::open(filepath)?;
		let mmap = unsafe { mmap::MmapChoice::auto().open(&file)? };

		Self::load_packed(mmap.as_ref())
	}

	/// Loads Gaussian splat from a slice of bytes in packed format.
	///
	/// `data` is gzip compressed, packed gaussian data.
	pub fn load_packed(data: &[u8]) -> Result<PackedGaussians> {
		let mut gz_decoder = flate2::read::GzDecoder::new(data);
		let mut res = String::new();

		gz_decoder.read_to_string(&mut res)?;

		Ok(res.try_into()?)
	}

	/// Loads Gaussian splat from a file in unpacked format.
	///
	/// `filepath` is uncompressed, unpacked gaussian data file.
	pub fn load_unpacked_from_file<F>(filepath: F) -> Result<GaussianCloud>
	where
		F: AsRef<Path>,
	{
		// mmap on macos isn't great according to ripgrep code
		if cfg!(target_os = "macos") {
			let infile = std::fs::read(filepath)?;

			return load_unpacked(&infile);
		}
		let file = File::open(filepath)?;
		let mmap = unsafe { mmap::MmapChoice::auto().open(&file)? };

		load_unpacked(mmap.as_ref())
	}

	/// Loads Gaussian splat from a slice of bytes in unpacked format.
	///
	/// `data` is uncompressed, unpacked gaussian data.
	pub fn load_unpacked(data: &[u8]) -> Result<GaussianCloud> {
		// Parse packed bytes into a GaussianCloud.
		unimplemented!()
	}

	// Saves Gaussian splat in packed format, returning a vector of bytes.
	pub fn save_as_spz_bytes(&self, _options: &PackOptions, _output: &mut Vec<u8>) -> bool {
		// Serialize 'gaussians' into packed bytes in 'output'.
		unimplemented!()
	}

	// Saves Gaussian splat in packed format to a file.
	pub fn save_as_spz(&self, _options: &PackOptions, _filename: &str) -> bool {
		unimplemented!()
	}

	// Saves Gaussian splat data in .ply format.
	pub fn save_splat_to_ply(&self, _options: &PackOptions, _filename: &str) -> bool {
		unimplemented!()
	}

	/// Convert between two coordinate systems (in-place).
	/// See [`CoordinateSystem`](src/types.rs).
	pub fn convert_coordinates(
		&mut self,
		from: crate::types::CoordinateSystem,
		to: crate::types::CoordinateSystem,
	) {
		if self.num_points == 0 {
			return;
		}
		let (x_match, y_match, z_match) = from.axes_match(to);

		let x = if x_match { 1.0_f64 } else { -1.0_f64 };
		let y = if y_match { 1.0_f64 } else { -1.0_f64 };
		let z = if z_match { 1.0_f64 } else { -1.0_f64 };

		let flip = CoordFlip {
			flip_p: [x, y, z],
			flip_q: [y * z, x * z, x * y],
			flip_sh: [
				y,         // 0
				z,         // 1
				x,         // 2
				x * y,     // 3
				y * z,     // 4
				1.0_f64,   // 5
				x * z,     // 6
				1.0_f64,   // 7
				y,         // 8
				x * y * z, // 9
				y,         // 10
				z,         // 11
				x,         // 12
				z,         // 13
				x,         // 14
			],
		};
		// Apply position flips (positions stored as [x,y,z, x,y,z, ...])
		for i in (0..self.positions.len()).step_by(3) {
			self.positions[i + 0] *= flip.flip_p[0];
			self.positions[i + 1] *= flip.flip_p[1];
			self.positions[i + 2] *= flip.flip_p[2];
		}
		// Apply quaternion flips: first three components are flipped, w (last component) left alone.
		// (C++ header used order x,y,z,w in the array.)
		for i in (0..self.rotations.len()).step_by(4) {
			self.rotations[i + 0] *= flip.flip_q[0];
			self.rotations[i + 1] *= flip.flip_q[1];
			self.rotations[i + 2] *= flip.flip_q[2];
			// rotations[i + 3] (w) unchanged
		}
		// Spherical harmonics: stored as [coeff0_r, coeff0_g, coeff0_b, coeff1_r, ...]
		// We invert coefficients that reference y/z axes according to flip_sh.
		let total_coeffs = if self.sh.len() >= 3 {
			self.sh.len() / 3
		} else {
			0
		};
		let num_points_usize = self.num_points.max(0) as usize;

		if num_points_usize == 0 || total_coeffs == 0 {
			return;
		}
		let coeffs_per_point = total_coeffs / num_points_usize;
		let mut idx = 0_usize;

		for _pt in 0..num_points_usize {
			for j in 0..coeffs_per_point {
				let f = flip.flip_sh[j];
				// multiply r,g,b for this coefficient
				self.sh[idx + 0] *= f;
				self.sh[idx + 1] *= f;
				self.sh[idx + 2] *= f;
				idx += 3;
			}
		}
	}

	/// Rotate 180 degrees about X axis (RUB <-> RDF).
	pub fn rotate_180_deg_about_x(&mut self) {
		self.convert_coordinates(
			crate::types::CoordinateSystem::RUB,
			crate::types::CoordinateSystem::RDF,
		);
	}

	/// Compute median ellipsoid volume.
	///
	/// Uses
	/// 	$V = \\tfrac{4}{3}\\pi\\,e^{x+y+z}$
	/// 		where x,y,z are log-scales.
	pub fn median_volume(&self) -> f64 {
		if self.num_points == 0 {
			return 0.01_f64;
		}
		let mut sums: Vec<f64> = Vec::with_capacity((self.scales.len() / 3).max(1));

		for i in (0..self.scales.len()).step_by(3) {
			let s = self.scales[i] + self.scales[i + 1] + self.scales[i + 2];

			sums.push(s);
		}
		if sums.is_empty() {
			return 0.01_f64;
		}
		sums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

		let median = sums[sums.len() / 2];

		(4.0_f64 / 3.0_f64) * std::f64::consts::PI * median.exp()
	}
}
