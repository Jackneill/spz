// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Python bindings for the SPZ Gaussian splatting file format.
//!
//! This crate provides Python bindings using PyO3 and numpy for efficient
//! array handling.

use numpy::{
	PyArray1, PyArray2, PyArrayMethods, PyReadonlyArray1, PyReadonlyArray2,
	PyUntypedArrayMethods,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::spz_rs;
use crate::spz_rs::header;

/// SPZ file format version.
///
/// Currently, only V2 and V3 are supported by this library.
/// V3 is the default and recommended version.
#[pyclass(eq, eq_int, frozen)]
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Version {
	/// Version 1 (unsupported).
	V1 = 1,
	/// Version 2.
	V2 = 2,
	/// Version 3.
	#[default]
	V3 = 3,
}

#[pymethods]
impl Version {
	pub fn __repr__(&self) -> &'static str {
		match self {
			Version::V1 => "Version.V1",
			Version::V2 => "Version.V2",
			Version::V3 => "Version.V3",
		}
	}

	pub fn __str__(&self) -> &'static str {
		match self {
			Version::V1 => "v1",
			Version::V2 => "v2",
			Version::V3 => "v3",
		}
	}
}

impl From<header::Version> for Version {
	fn from(v: header::Version) -> Self {
		match v {
			header::Version::V1 => Version::V1,
			header::Version::V2 => Version::V2,
			header::Version::V3 => Version::V3,
		}
	}
}

impl From<Version> for header::Version {
	fn from(v: Version) -> Self {
		match v {
			Version::V1 => header::Version::V1,
			Version::V2 => header::Version::V2,
			Version::V3 => header::Version::V3,
		}
	}
}

/// SPZ file header.
///
/// Contains metadata about a Gaussian Splat file, such as the number
/// of points, version, spherical harmonics degree, and flags.
///
/// Headers can be read from files or bytes without loading the full
/// splat data, which is useful for quick inspection.
///
/// # Examples
///
/// Read header from a file:
///
/// ```python
/// header = spz.Header.from_file("scene.spz")
/// print(f"Version: {header.version}, Points: {header.num_points}")
/// ```
///
/// Read header from bytes:
///
/// ```python
/// with open("scene.spz", "rb") as f:
///     data = f.read()
/// header = spz.Header.from_bytes(data)
/// ```
#[pyclass(frozen)]
#[derive(Clone)]
pub struct Header {
	inner: header::Header,
}

#[pymethods]
impl Header {
	/// Reads a header from an SPZ file without loading the full splat data.
	///
	/// This is efficient for quickly inspecting SPZ file metadata.
	///
	/// # Args
	///
	/// * `path` - Path to the SPZ file.
	///
	/// # Returns
	///
	/// The parsed header.
	///
	/// # Errors
	///
	/// Returns `ValueError` if the file cannot be read or the header is invalid.
	#[staticmethod]
	pub fn from_file(path: &str) -> PyResult<Self> {
		let inner = header::Header::from_file(path).map_err(|e| {
			PyValueError::new_err(format!("Failed to read SPZ header: {}", e))
		})?;
		Ok(Self { inner })
	}

	/// Reads a header from compressed SPZ bytes.
	///
	/// # Args
	///
	/// * `data` - The compressed SPZ file contents as bytes.
	///
	/// # Returns
	///
	/// The parsed header.
	///
	/// # Errors
	///
	/// Returns `ValueError` if the data is invalid or the header fails
	/// validation.
	#[staticmethod]
	pub fn from_bytes(data: &[u8]) -> PyResult<Self> {
		let inner = header::Header::from_compressed_bytes(data).map_err(|e| {
			PyValueError::new_err(format!("Failed to parse SPZ header: {}", e))
		})?;
		Ok(Self { inner })
	}

	/// The SPZ format version.
	#[getter]
	pub fn version(&self) -> Version {
		self.inner.version.into()
	}

	/// The number of Gaussian points.
	#[getter]
	pub fn num_points(&self) -> i32 {
		self.inner.num_points
	}

	/// The spherical harmonics degree (0-3).
	#[getter]
	pub fn sh_degree(&self) -> u8 {
		self.inner.spherical_harmonics_degree
	}

	/// The number of fractional bits in position encoding.
	///
	/// Standard value is 12, giving ~0.25mm resolution.
	#[getter]
	pub fn fractional_bits(&self) -> u8 {
		self.inner.fractional_bits
	}

	/// Whether the splat was trained with antialiasing.
	#[getter]
	pub fn antialiased(&self) -> bool {
		self.inner.flags.is_antialiased()
	}

	/// Validates the header.
	///
	/// Checks magic number, version, SH degree range, num_points,
	/// flags, and reserved bytes.
	///
	/// # Returns
	///
	/// `True` if the header is valid.
	pub fn is_valid(&self) -> bool {
		self.inner.is_valid()
	}

	/// Returns a detailed, human-readable summary of the header.
	pub fn pretty_fmt(&self) -> String {
		self.inner.pretty_fmt()
	}

	pub fn __repr__(&self) -> String {
		format!(
			"Header(version={}, num_points={}, sh_degree={}, fractional_bits={}, antialiased={})",
			self.inner.version,
			self.inner.num_points,
			self.inner.spherical_harmonics_degree,
			self.inner.fractional_bits,
			self.inner.flags.is_antialiased(),
		)
	}

	pub fn __str__(&self) -> String {
		format!("{}", self.inner)
	}
}

#[pyclass(eq, frozen)]
#[derive(Clone, PartialEq)]
pub struct CoordinateSystem {
	inner: spz_rs::coord::CoordinateSystem,
}

#[pymethods]
impl CoordinateSystem {
	/// Left Down Back
	#[classattr]
	#[allow(non_snake_case)]
	pub fn LDB() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::LeftDownBack,
		}
	}

	/// Right Down Back
	#[classattr]
	#[allow(non_snake_case)]
	pub fn RDB() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::RightDownBack,
		}
	}

	/// Left Up Back
	#[classattr]
	#[allow(non_snake_case)]
	pub fn LUB() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::LeftUpBack,
		}
	}

	/// Right Up Back (Three.js coordinate system)
	#[classattr]
	#[allow(non_snake_case)]
	pub fn RUB() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::RightUpBack,
		}
	}

	/// Left Down Front
	#[classattr]
	#[allow(non_snake_case)]
	pub fn LDF() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::LeftDownFront,
		}
	}

	/// Right Down Front (PLY file coordinate system)
	#[classattr]
	#[allow(non_snake_case)]
	pub fn RDF() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::RightDownFront,
		}
	}

	/// Left Up Front (GLB/glTF coordinate system)
	#[classattr]
	#[allow(non_snake_case)]
	pub fn LUF() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::LeftUpFront,
		}
	}

	/// Right Up Front (Unity coordinate system)
	#[classattr]
	#[allow(non_snake_case)]
	pub fn RUF() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::RightUpFront,
		}
	}

	/// Unspecified coordinate system (no conversion)
	#[classattr]
	#[allow(non_snake_case)]
	pub fn UNSPECIFIED() -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::Unspecified,
		}
	}

	/// Parse a coordinate system from a string.
	///
	/// Accepts 3-letter abbreviations (e.g. `"RDF"`, `"LUF"`) and
	/// full names (e.g. `"Right-Down-Front"`, `"LEFT_UP_FRONT"`).
	/// Case-insensitive. Returns `UNSPECIFIED` for unrecognized strings.
	///
	/// # Args
	///
	/// * `coord` - String representation of the coordinate system.
	///
	/// # Returns
	///
	/// The parsed coordinate system.
	#[staticmethod]
	pub fn from_str(coord: &str) -> Self {
		Self {
			inner: spz_rs::coord::CoordinateSystem::from(coord),
		}
	}

	/// Returns the short 3-letter abbreviation (e.g. `"RDF"`, `"LUF"`).
	///
	/// Returns `"UNSPECIFIED"` for the unspecified coordinate system.
	#[getter]
	pub fn short_name(&self) -> &'static str {
		self.inner.as_short_str()
	}

	#[inline]
	pub fn __repr__(&self) -> String {
		self.__str__()
	}

	#[inline]
	pub fn __str__(&self) -> String {
		format!("{}", self.inner)
	}
}

/// Bounding box of a Gaussian splat.
#[pyclass]
#[derive(Clone)]
pub struct BoundingBox {
	inner: spz_rs::gaussian_splat::BoundingBox,
}

#[pymethods]
impl BoundingBox {
	pub fn __repr__(&self) -> String {
		format!(
			"BoundingBox(x=[{}, {}], y=[{}, {}], z=[{}, {}])",
			self.inner.min_x,
			self.inner.max_x,
			self.inner.min_y,
			self.inner.max_y,
			self.inner.min_z,
			self.inner.max_z
		)
	}

	/// Minimum X coordinate.
	#[getter]
	pub fn min_x(&self) -> f32 {
		self.inner.min_x
	}

	/// Maximum X coordinate.
	#[getter]
	pub fn max_x(&self) -> f32 {
		self.inner.max_x
	}

	/// Minimum Y coordinate.
	#[getter]
	pub fn min_y(&self) -> f32 {
		self.inner.min_y
	}

	/// Maximum Y coordinate.
	#[getter]
	pub fn max_y(&self) -> f32 {
		self.inner.max_y
	}

	/// Minimum Z coordinate.
	#[getter]
	pub fn min_z(&self) -> f32 {
		self.inner.min_z
	}

	/// Maximum Z coordinate.
	#[getter]
	pub fn max_z(&self) -> f32 {
		self.inner.max_z
	}

	/// Returns the size (extent) of the bounding box in each dimension.
	///
	/// Returns a tuple of `(width, height, depth)`.
	#[getter]
	pub fn size(&self) -> (f32, f32, f32) {
		self.inner.size()
	}

	/// Returns the center of the bounding box.
	///
	/// Returns a tuple of `(x, y, z)` center coordinates.
	#[getter]
	pub fn center(&self) -> (f32, f32, f32) {
		self.inner.center()
	}
}

/// A 3D Gaussian Splat point cloud.
///
/// This class represents a collection of 3D Gaussians used for
/// Gaussian splatting rendering. Each Gaussian has a position,
/// rotation, scale, color, alpha (opacity), and spherical harmonics
/// coefficients for view-dependent appearance.
///
/// All array data is returned as numpy arrays for efficient processing.
///
/// # Examples
///
/// Load from file:
///
/// ```python
/// splat = spz.GaussianSplat.load("scene.spz")
/// print(f"Loaded {splat.num_points} gaussians")
/// # Access as numpy arrays
/// positions = splat.positions  # shape: (num_points, 3)
/// ```
///
/// Create from numpy arrays:
///
/// ```python
/// import numpy as np
///
/// splat = spz.GaussianSplat(
///     positions=np.zeros((100, 3), dtype=np.float32),
///     scales=np.full((100, 3), -5.0, dtype=np.float32),
///     rotations=np.tile([1, 0, 0, 0], (100, 1)).astype(np.float32),
///     alphas=np.zeros(100, dtype=np.float32),
///     colors=np.zeros((100, 3), dtype=np.float32),
/// )
/// ```
#[pyclass]
#[derive(Clone)]
pub struct GaussianSplat {
	inner: spz_rs::gaussian_splat::GaussianSplat,
}

#[pymethods]
impl GaussianSplat {
	/// Creates a new `GaussianSplat` from numpy arrays.
	///
	/// # Args
	///
	/// * `positions` - `(N, 3)` array of `(x, y, z)` positions.
	/// * `scales` - `(N, 3)` array of `(x, y, z)` log-scale values.
	/// * `rotations` - `(N, 4)` array of `(w, x, y, z)` quaternion rotations.
	/// * `alphas` - `(N,)` array of inverse-sigmoid opacity values.
	/// * `colors` - `(N, 3)` array of `(r, g, b)` SH0 color values.
	/// * `sh_degree` - Spherical harmonics degree (0-3). Defaults to 0.
	/// * `spherical_harmonics` - `(N, sh_dim, 3)` array of SH coefficients.
	///   Defaults to `None`.
	/// * `antialiased` - Whether the splat was trained with antialiasing.
	///   Defaults to `false`.
	#[new]
	#[pyo3(signature = (
		/,
		positions,
		scales,
		rotations,
		alphas,
		colors,
		sh_degree=0,
		spherical_harmonics=None,
		antialiased=false
	))]
	pub fn new(
		positions: PyReadonlyArray2<f32>,
		scales: PyReadonlyArray2<f32>,
		rotations: PyReadonlyArray2<f32>,
		alphas: PyReadonlyArray1<f32>,
		colors: PyReadonlyArray2<f32>,
		sh_degree: u8,
		spherical_harmonics: Option<PyReadonlyArray2<f32>>,
		antialiased: bool,
	) -> PyResult<Self> {
		// Get num_points from the first dimension of positions array (shape is (N, 3))
		let num_points = positions.shape()[0];
		let positions_vec = positions.as_slice()?.to_vec();
		let scales_vec = scales.as_slice()?.to_vec();
		let rotations_vec = rotations.as_slice()?.to_vec();
		let alphas_vec = alphas.as_slice()?.to_vec();
		let colors_vec = colors.as_slice()?.to_vec();

		let spherical_harmonics_vec = if let Some(v) = spherical_harmonics {
			v.as_slice()?.to_vec()
		} else {
			Vec::new()
		};
		Ok(Self {
			inner: spz_rs::gaussian_splat::GaussianSplat {
				header: header::Header {
					num_points: num_points as i32,
					spherical_harmonics_degree: sh_degree,
					flags: if antialiased {
						header::Flags::ANTIALIASED
					} else {
						header::Flags::none()
					},
					..Default::default()
				},
				positions: positions_vec,
				scales: scales_vec,
				rotations: rotations_vec,
				alphas: alphas_vec,
				colors: colors_vec,
				spherical_harmonics: spherical_harmonics_vec,
			},
		})
	}

	/// Loads a `GaussianSplat` from an SPZ file.
	///
	/// # Args
	///
	/// * `path` - Path to the SPZ file.
	/// * `coordinate_system` - The coordinate system to convert the data to
	/// 	from the one it is stored.
	/// 	Defaults to `UNSPECIFIED` (no conversion).
	///
	/// # Returns
	///
	/// The loaded Gaussian splat.
	///
	/// # Errors
	///
	/// Returns `ValueError` if the file cannot be read or is invalid.
	#[staticmethod]
	#[pyo3(signature = (path, coordinate_system=CoordinateSystem::UNSPECIFIED()))]
	pub fn load(path: &str, coordinate_system: CoordinateSystem) -> PyResult<Self> {
		let opts = spz_rs::gaussian_splat::LoadOptions {
			coord_sys: coordinate_system.inner,
		};
		let inner = spz_rs::gaussian_splat::GaussianSplat::load_with(path, &opts).map_err(
			|e| PyValueError::new_err(format!("Failed to load SPZ file: {}", e)),
		)?;

		Ok(Self { inner })
	}

	/// Loads a `GaussianSplat` from bytes.
	///
	/// # Args
	///
	/// * `data` - The SPZ file contents as bytes.
	/// * `coordinate_system` - The coordinate system to convert the data to
	/// 	from the one it is stored.
	/// 	Defaults to `UNSPECIFIED` (no conversion).
	///
	/// # Returns
	///
	/// The loaded Gaussian splat.
	#[staticmethod]
	#[pyo3(signature = (data, coordinate_system=CoordinateSystem::UNSPECIFIED()))]
	pub fn from_bytes(data: &[u8], coordinate_system: CoordinateSystem) -> PyResult<Self> {
		let opts = spz_rs::gaussian_splat::LoadOptions {
			coord_sys: coordinate_system.inner,
		};
		let packed =
			spz_rs::packed::PackedGaussianSplat::from_bytes(data).map_err(|e| {
				PyValueError::new_err(format!("Failed to parse SPZ data: {}", e))
			})?;
		let inner = spz_rs::gaussian_splat::GaussianSplat::new_from_packed_gaussians(
			&packed, &opts,
		)
		.map_err(|e| PyValueError::new_err(format!("Failed to unpack SPZ data: {}", e)))?;

		Ok(Self { inner })
	}

	/// Saves the `GaussianSplat` to an SPZ file.
	///
	/// # Args
	///
	/// * `path` - Path to save the SPZ file.
	/// * `coordinate_system` - The coordinate system to save the data in.
	/// 	Defaults to `UNSPECIFIED` (no conversion).
	#[pyo3(signature = (path, coordinate_system=CoordinateSystem::UNSPECIFIED()))]
	pub fn save(&self, path: &str, coordinate_system: CoordinateSystem) -> PyResult<()> {
		let pack_opts = spz_rs::gaussian_splat::SaveOptions {
			coord_sys: coordinate_system.inner,
		};
		self.inner.save(path, &pack_opts).map_err(|e| {
			PyValueError::new_err(format!("Failed to save SPZ file: {}", e))
		})
	}

	/// Serializes the `GaussianSplat` to bytes.
	///
	/// # Args
	///
	/// * `coordinate_system` - The coordinate system to serialize the data in.
	/// 	Defaults to `UNSPECIFIED` (no conversion).
	///
	/// # Returns
	///
	/// The SPZ file contents as bytes.
	#[pyo3(signature = (coordinate_system=CoordinateSystem::UNSPECIFIED()))]
	pub fn to_bytes<'py>(
		&self,
		py: Python<'py>,
		coordinate_system: CoordinateSystem,
	) -> PyResult<Bound<'py, PyBytes>> {
		let pack_opts = spz_rs::gaussian_splat::SaveOptions {
			coord_sys: coordinate_system.inner,
		};
		let bytes = self
			.inner
			.serialize_to_packed_bytes(&pack_opts)
			.map_err(|e| {
				PyValueError::new_err(format!("Failed to serialize SPZ: {}", e))
			})?;

		Ok(PyBytes::new(py, &bytes))
	}

	/// Converts coordinates to a different coordinate system.
	///
	/// # Args
	///
	/// * `source` - The current coordinate system of the data.
	/// * `target` - The target coordinate system.
	#[inline]
	pub fn convert_coordinates(&mut self, source: CoordinateSystem, target: CoordinateSystem) {
		self.inner.convert_coordinates(source.inner, target.inner);
	}

	/// Returns the number of Gaussian points.
	#[inline]
	#[getter]
	pub fn num_points(&self) -> i32 {
		self.inner.header.num_points
	}

	/// Returns the spherical harmonics degree (0-3).
	#[inline]
	#[getter]
	pub fn sh_degree(&self) -> u8 {
		self.inner.header.spherical_harmonics_degree
	}

	/// Returns whether the splat was trained with antialiasing.
	#[inline]
	#[getter]
	pub fn antialiased(&self) -> bool {
		self.inner.header.flags.is_antialiased()
	}

	/// Returns an `(N, 3)` array of `(x, y, z)` positions.
	#[inline]
	#[getter]
	pub fn positions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f32>>> {
		let n = self.inner.header.num_points as usize;
		let arr = PyArray1::from_slice(py, &self.inner.positions);
		let reshaped = arr.reshape([n, 3])?;

		Ok(reshaped)
	}

	/// Returns an `(N, 3)` array of `(x, y, z)` log-scale values.
	#[getter]
	pub fn scales<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f32>>> {
		let n = self.inner.header.num_points as usize;
		let arr = PyArray1::from_slice(py, &self.inner.scales);
		let reshaped = arr.reshape([n, 3])?;

		Ok(reshaped)
	}

	/// Returns an `(N, 4)` array of `(w, x, y, z)` quaternion rotations.
	#[getter]
	pub fn rotations<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f32>>> {
		let n = self.inner.header.num_points as usize;
		let arr = PyArray1::from_slice(py, &self.inner.rotations);
		let reshaped = arr.reshape([n, 4])?;

		Ok(reshaped)
	}

	/// Returns an `(N,)` array of inverse-sigmoid opacity values.
	#[getter]
	pub fn alphas<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f32>> {
		PyArray1::from_slice(py, &self.inner.alphas)
	}

	/// Returns an `(N, 3)` array of `(r, g, b)` SH0 color values.
	#[getter]
	pub fn colors<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f32>>> {
		let n = self.inner.header.num_points as usize;
		let arr = PyArray1::from_slice(py, &self.inner.colors);
		let reshaped = arr.reshape([n, 3])?;

		Ok(reshaped)
	}

	/// Returns an `(N, sh_dim * 3)` array of spherical harmonics coefficients.
	///
	/// The `sh_dim` depends on `sh_degree`: 0→0, 1→3, 2→8, 3→15.
	///
	/// Returns an empty `(N, 0)` array if `sh_degree` is 0.
	#[getter]
	pub fn spherical_harmonics<'py>(
		&self,
		py: Python<'py>,
	) -> PyResult<Bound<'py, PyArray2<f32>>> {
		let n = self.inner.header.num_points as usize;

		let sh_dim =
			spz_rs::math::dim_for_degree(self.inner.header.spherical_harmonics_degree);
		if sh_dim == 0 {
			// Return empty (N, 0) array
			let arr = PyArray1::<f32>::from_slice(py, &[]);
			let reshaped = arr.reshape([n, 0])?;

			Ok(reshaped)
		} else {
			let arr = PyArray1::from_slice(py, &self.inner.spherical_harmonics);
			// Return as (N, sh_dim * 3) for simplicity
			let reshaped = arr.reshape([n, sh_dim as usize * 3])?;

			Ok(reshaped)
		}
	}

	/// Returns the bounding box of the splat.
	#[getter]
	pub fn bbox(&self) -> BoundingBox {
		let inner = self.inner.bbox();

		BoundingBox { inner }
	}

	/// Returns the median ellipsoid volume of the Gaussians.
	///
	/// This is useful for understanding the typical size of the
	/// Gaussians in the point cloud.
	#[getter]
	pub fn median_volume(&self) -> f32 {
		self.inner.median_volume()
	}

	/// Returns the file header.
	#[getter]
	pub fn header(&self) -> Header {
		Header {
			inner: self.inner.header,
		}
	}

	/// Returns the SPZ format version.
	#[getter]
	pub fn version(&self) -> Version {
		self.inner.header.version.into()
	}

	/// Returns the number of fractional bits used in position encoding.
	///
	/// Standard value is 12, giving ~0.25mm resolution.
	#[getter]
	pub fn fractional_bits(&self) -> u8 {
		self.inner.header.fractional_bits
	}

	/// Validates that all internal arrays have consistent sizes.
	///
	/// Checks that all arrays match the expected dimensions for
	/// the given `num_points` and `sh_degree`.
	///
	/// # Returns
	///
	/// `True` if all sizes are valid.
	pub fn check_sizes(&self) -> bool {
		self.inner.check_sizes()
	}

	/// Returns a detailed, human-readable summary of the splat.
	///
	/// Includes header information, median volume, and bounding box.
	pub fn pretty_fmt(&self) -> String {
		self.inner.pretty_fmt()
	}

	pub fn __repr__(&self) -> String {
		format!(
			"GaussianSplat(num_points={}, sh_degree={}, antialiased={}, ..)",
			self.inner.header.num_points,
			self.inner.header.spherical_harmonics_degree,
			self.inner.header.flags.is_antialiased()
		)
	}

	pub fn __str__(&self) -> String {
		self.inner.to_string()
	}

	pub fn __len__(&self) -> usize {
		self.inner.header.num_points as usize
	}
}

/// Loads a [`GaussianSplat`] from an SPZ file.
///
/// This is a convenience function equivalent to [`GaussianSplat::load`].
///
/// # Args
///
/// * `path` - Path to the SPZ file.
/// * `coordinate_system` - Target coordinate system for the loaded data.
///
/// # Returns
///
/// The loaded Gaussian Splat.
#[inline]
#[pyfunction]
#[pyo3(signature = (path, coordinate_system=CoordinateSystem::UNSPECIFIED()))]
pub fn load(path: &str, coordinate_system: CoordinateSystem) -> PyResult<GaussianSplat> {
	GaussianSplat::load(path, coordinate_system)
}

/// Reads only the header from an SPZ file without loading the full data.
///
/// This is a convenience function equivalent to [`Header::from_file`].
///
/// # Args
///
/// * `path` - Path to the SPZ file.
///
/// # Returns
///
/// The file header.
#[inline]
#[pyfunction]
pub fn read_header(path: &str) -> PyResult<Header> {
	Header::from_file(path)
}

/// SPZ - Gaussian Splat file format library.
///
/// A fast, efficient library for reading and writing SPZ files,
/// which store 3D Gaussian Splat point clouds in a compressed format.
///
/// All array data is returned as numpy arrays for efficient processing.
///
/// # Exports
///
/// * [`GaussianSplat`] - A 3D Gaussian Splat point cloud.
/// * [`CoordinateSystem`] - Enumeration of coordinate systems (RUB, RDF, etc.).
/// * [`BoundingBox`] - Axis-aligned bounding box.
/// * [`load`] - Load a GaussianSplat from an SPZ file.
///
/// # Examples
///
/// ```python
/// import spz
///
/// splat = spz.load("scene.spz")
/// print(f"Loaded {len(splat)} gaussians")
///
/// positions = splat.positions  # numpy array (N, 3)
/// splat.save("output.spz")
/// ```
#[pymodule]
pub fn spz(m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_class::<GaussianSplat>()?;
	m.add_class::<CoordinateSystem>()?;
	m.add_class::<BoundingBox>()?;
	m.add_class::<Header>()?;
	m.add_class::<Version>()?;
	m.add_function(wrap_pyfunction!(load, m)?)?;
	m.add_function(wrap_pyfunction!(read_header, m)?)?;

	Ok(())
}
