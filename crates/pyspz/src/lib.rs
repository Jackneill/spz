// SPDX-License-Identifier: Apache-2.0 OR MIT

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};

// Import the spz crate with a different name to avoid conflict with the pymodule.
use ::spz as spz_rs;

/// Convert spz::CoordinateSystem to/from our Python enum.
fn coord_sys_from_str(s: &str) -> PyResult<spz_rs::CoordinateSystem> {
	match s.to_uppercase().as_str() {
		"LDB" => Ok(spz_rs::CoordinateSystem::LDB),
		"RDB" => Ok(spz_rs::CoordinateSystem::RDB),
		"LUB" => Ok(spz_rs::CoordinateSystem::LUB),
		"RUB" => Ok(spz_rs::CoordinateSystem::RUB),
		"LDF" => Ok(spz_rs::CoordinateSystem::LDF),
		"RDF" => Ok(spz_rs::CoordinateSystem::RDF),
		"LUF" => Ok(spz_rs::CoordinateSystem::LUF),
		"RUF" => Ok(spz_rs::CoordinateSystem::RUF),
		"UNSPECIFIED" | "" => Ok(spz_rs::CoordinateSystem::UNSPECIFIED),
		_ => Err(PyValueError::new_err(format!(
			"Invalid coordinate system: '{}'. Valid options: LDB, RDB, LUB, RUB, LDF, RDF, LUF, RUF, UNSPECIFIED",
			s
		))),
	}
}

fn coord_sys_to_str(cs: &spz_rs::CoordinateSystem) -> &'static str {
	match cs {
		spz_rs::CoordinateSystem::LDB => "LDB",
		spz_rs::CoordinateSystem::RDB => "RDB",
		spz_rs::CoordinateSystem::LUB => "LUB",
		spz_rs::CoordinateSystem::RUB => "RUB",
		spz_rs::CoordinateSystem::LDF => "LDF",
		spz_rs::CoordinateSystem::RDF => "RDF",
		spz_rs::CoordinateSystem::LUF => "LUF",
		spz_rs::CoordinateSystem::RUF => "RUF",
		spz_rs::CoordinateSystem::UNSPECIFIED => "UNSPECIFIED",
	}
}

/// Coordinate system enum.
#[pyclass(eq, frozen)]
#[derive(Clone, PartialEq)]
pub struct CoordinateSystem {
	inner: spz_rs::CoordinateSystem,
}

#[pymethods]
impl CoordinateSystem {
	#[new]
	#[pyo3(signature = (name="UNSPECIFIED"))]
	fn new(name: &str) -> PyResult<Self> {
		Ok(Self {
			inner: coord_sys_from_str(name)?,
		})
	}

	/// Left Down Back
	#[classattr]
	#[allow(non_snake_case)]
	fn LDB() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::LDB,
		}
	}

	/// Right Down Back
	#[classattr]
	#[allow(non_snake_case)]
	fn RDB() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::RDB,
		}
	}

	/// Left Up Back
	#[classattr]
	#[allow(non_snake_case)]
	fn LUB() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::LUB,
		}
	}

	/// Right Up Back (Three.js coordinate system)
	#[classattr]
	#[allow(non_snake_case)]
	fn RUB() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::RUB,
		}
	}

	/// Left Down Front
	#[classattr]
	#[allow(non_snake_case)]
	fn LDF() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::LDF,
		}
	}

	/// Right Down Front (PLY file coordinate system)
	#[classattr]
	#[allow(non_snake_case)]
	fn RDF() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::RDF,
		}
	}

	/// Left Up Front (GLB/glTF coordinate system)
	#[classattr]
	#[allow(non_snake_case)]
	fn LUF() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::LUF,
		}
	}

	/// Right Up Front (Unity coordinate system)
	#[classattr]
	#[allow(non_snake_case)]
	fn RUF() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::RUF,
		}
	}

	/// Unspecified coordinate system (no conversion)
	#[classattr]
	#[allow(non_snake_case)]
	fn UNSPECIFIED() -> Self {
		Self {
			inner: spz_rs::CoordinateSystem::UNSPECIFIED,
		}
	}

	fn __repr__(&self) -> String {
		format!("CoordinateSystem.{}", coord_sys_to_str(&self.inner))
	}

	fn __str__(&self) -> &'static str {
		coord_sys_to_str(&self.inner)
	}
}

/// Bounding box of a Gaussian splat.
#[pyclass(get_all)]
#[derive(Clone)]
pub struct BoundingBox {
	pub min_x: f32,
	pub max_x: f32,
	pub min_y: f32,
	pub max_y: f32,
	pub min_z: f32,
	pub max_z: f32,
}

#[pymethods]
impl BoundingBox {
	fn __repr__(&self) -> String {
		format!(
			"BoundingBox(x=[{:.6}, {:.6}], y=[{:.6}, {:.6}], z=[{:.6}, {:.6}])",
			self.min_x, self.max_x, self.min_y, self.max_y, self.min_z, self.max_z
		)
	}

	/// Get the size (extent) of the bounding box in each dimension.
	///
	/// Returns:
	///     A tuple of (width, height, depth).
	#[getter]
	fn size(&self) -> (f32, f32, f32) {
		(
			self.max_x - self.min_x,
			self.max_y - self.min_y,
			self.max_z - self.min_z,
		)
	}

	/// Get the center of the bounding box.
	///
	/// Returns:
	///     A tuple of (x, y, z) center coordinates.
	#[getter]
	fn center(&self) -> (f32, f32, f32) {
		(
			(self.min_x + self.max_x) / 2.0,
			(self.min_y + self.max_y) / 2.0,
			(self.min_z + self.max_z) / 2.0,
		)
	}
}

/// A 3D Gaussian Splat point cloud.
///
/// This class represents a collection of 3D Gaussians used for
/// Gaussian splatting rendering. Each Gaussian has a position,
/// rotation, scale, color, alpha (opacity), and spherical harmonics
/// coefficients for view-dependent appearance.
///
/// Example:
///     Load from file::
///
///         splat = spz.GaussianSplat.load("scene.spz")
///         print(f"Loaded {splat.num_points} gaussians")
///
///     Create from data::
///
///         splat = spz.GaussianSplat(
///             positions=[0.0, 0.0, 0.0, 1.0, 0.0, 0.0],  # 2 points
///             scales=[-5.0] * 6,
///             rotations=[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
///             alphas=[0.0, 0.0],
///             colors=[0.0] * 6,
///             sh_degree=0
///         )
#[pyclass]
#[derive(Clone)]
pub struct GaussianSplat {
	inner: spz_rs::GaussianSplat,
}

#[pymethods]
impl GaussianSplat {
	/// Create a new GaussianSplat from arrays.
	///
	/// Args:
	///     positions: Flattened (x, y, z) positions. Length must be num_points * 3.
	///     scales: Flattened (x, y, z) log-scale values. Length must be num_points * 3.
	///     rotations: Flattened (x, y, z, w) quaternion rotations. Length must be
	///         num_points * 4.
	///     alphas: Inverse-sigmoid opacity values. Length must be num_points.
	///     colors: Flattened (r, g, b) SH0 color values. Length must be num_points * 3.
	///     sh_degree: Spherical harmonics degree (0-3). Defaults to 0.
	///     spherical_harmonics: Flattened SH coefficients. Length depends on sh_degree.
	///         Defaults to None.
	///     antialiased: Whether the splat was trained with antialiasing. Defaults to False.
	#[new]
	#[pyo3(signature = (
		positions,
		scales,
		rotations,
		alphas,
		colors,
		sh_degree=0,
		spherical_harmonics=None,
		antialiased=false
	))]
	fn new(
		positions: Vec<f32>,
		scales: Vec<f32>,
		rotations: Vec<f32>,
		alphas: Vec<f32>,
		colors: Vec<f32>,
		sh_degree: i32,
		spherical_harmonics: Option<Vec<f32>>,
		antialiased: bool,
	) -> PyResult<Self> {
		// Validate sh_degree
		if sh_degree < 0 || sh_degree > 3 {
			return Err(PyValueError::new_err(format!(
				"sh_degree must be between 0 and 3, got {}",
				sh_degree
			)));
		}
		// Calculate expected sizes
		if positions.len() % 3 != 0 {
			return Err(PyValueError::new_err(
				"positions length must be a multiple of 3",
			));
		}
		let num_points = positions.len() / 3;

		// Validate array sizes
		if scales.len() != num_points * 3 {
			return Err(PyValueError::new_err(format!(
				"scales length must be {}, got {}",
				num_points * 3,
				scales.len()
			)));
		}
		if rotations.len() != num_points * 4 {
			return Err(PyValueError::new_err(format!(
				"rotations length must be {}, got {}",
				num_points * 4,
				rotations.len()
			)));
		}
		if alphas.len() != num_points {
			return Err(PyValueError::new_err(format!(
				"alphas length must be {}, got {}",
				num_points,
				alphas.len()
			)));
		}
		if colors.len() != num_points * 3 {
			return Err(PyValueError::new_err(format!(
				"colors length must be {}, got {}",
				num_points * 3,
				colors.len()
			)));
		}
		// Calculate expected SH size
		let sh_dim = match sh_degree {
			0 => 0,
			1 => 3,
			2 => 8,
			3 => 15,
			_ => unreachable!(),
		};
		let expected_sh_len = num_points * sh_dim * 3;

		let spherical_harmonics = match spherical_harmonics {
			Some(sh) => {
				if sh.len() != expected_sh_len {
					return Err(PyValueError::new_err(format!(
						"spherical_harmonics length must be {} for sh_degree={}, got {}",
						expected_sh_len, sh_degree, sh.len()
					)));
				}
				sh
			},
			None => vec![0.0_f32; expected_sh_len],
		};

		Ok(Self {
			inner: spz_rs::GaussianSplat {
				num_points: num_points as i32,
				spherical_harmonics_degree: sh_degree,
				antialiased,
				positions,
				scales,
				rotations,
				alphas,
				colors,
				spherical_harmonics,
			},
		})
	}

	/// Load a GaussianSplat from an SPZ file.
	///
	/// Args:
	///     path: Path to the SPZ file.
	///     coordinate_system: Target coordinate system for the loaded data.
	///         Defaults to UNSPECIFIED (no conversion).
	///
	/// Returns:
	///     The loaded Gaussian splat.
	///
	/// Raises:
	///     ValueError: If the file cannot be read or is invalid.
	#[staticmethod]
	#[pyo3(signature = (path, coordinate_system=None))]
	fn load(path: &str, coordinate_system: Option<CoordinateSystem>) -> PyResult<Self> {
		let coord_sys = coordinate_system
			.map(|cs| cs.inner)
			.unwrap_or(spz_rs::CoordinateSystem::UNSPECIFIED);

		let unpack_opts = spz_rs::UnpackOptions {
			to_coord_sys: coord_sys,
		};
		let inner = spz_rs::GaussianSplat::load_packed_from_file(path, &unpack_opts)
			.map_err(|e| PyValueError::new_err(format!("Failed to load SPZ file: {}", e)))?;

		Ok(Self { inner })
	}

	/// Load a GaussianSplat from bytes.
	///
	/// Args:
	///     data: The SPZ file contents as bytes.
	///     coordinate_system: Target coordinate system for the loaded data.
	///         Defaults to UNSPECIFIED (no conversion).
	///
	/// Returns:
	///     The loaded Gaussian splat.
	#[staticmethod]
	#[pyo3(signature = (data, coordinate_system=None))]
	fn from_bytes(data: &[u8], coordinate_system: Option<CoordinateSystem>) -> PyResult<Self> {
		let coord_sys = coordinate_system
			.map(|cs| cs.inner)
			.unwrap_or(spz_rs::CoordinateSystem::UNSPECIFIED);

		let unpack_opts = spz_rs::UnpackOptions {
			to_coord_sys: coord_sys,
		};
		let packed = spz_rs::GaussianSplat::load_packed(data)
			.map_err(|e| PyValueError::new_err(format!("Failed to parse SPZ data: {}", e)))?;

		let inner = spz_rs::GaussianSplat::new_from_packed_gaussians(&packed, &unpack_opts)
			.map_err(|e| PyValueError::new_err(format!("Failed to unpack SPZ data: {}", e)))?;

		Ok(Self { inner })
	}

	/// Save the GaussianSplat to an SPZ file.
	///
	/// Args:
	///     path: Path to save the SPZ file.
	///     from_coordinate_system: Source coordinate system of the data.
	///         Defaults to UNSPECIFIED (no conversion).
	#[pyo3(signature = (path, from_coordinate_system=None))]
	fn save(&self, path: &str, from_coordinate_system: Option<CoordinateSystem>) -> PyResult<()> {
		let coord_sys = from_coordinate_system
			.map(|cs| cs.inner)
			.unwrap_or(spz_rs::CoordinateSystem::UNSPECIFIED);

		let pack_opts = spz_rs::PackOptions { from: coord_sys };
		self.inner
			.save_as_packed(path, &pack_opts)
			.map_err(|e| PyValueError::new_err(format!("Failed to save SPZ file: {}", e)))
	}

	/// Serialize the GaussianSplat to bytes.
	///
	/// Args:
	///     from_coordinate_system: Source coordinate system of the data.
	///         Defaults to UNSPECIFIED (no conversion).
	///
	/// Returns:
	///     The SPZ file contents as bytes.
	#[pyo3(signature = (from_coordinate_system=None))]
	fn to_bytes<'py>(
		&self,
		py: Python<'py>,
		from_coordinate_system: Option<CoordinateSystem>,
	) -> PyResult<Bound<'py, PyBytes>> {
		let coord_sys = from_coordinate_system
			.map(|cs| cs.inner)
			.unwrap_or(spz_rs::CoordinateSystem::UNSPECIFIED);

		let pack_opts = spz_rs::PackOptions { from: coord_sys };
		let bytes = self
			.inner
			.serialize_as_packed_bytes(&pack_opts)
			.map_err(|e| PyValueError::new_err(format!("Failed to serialize SPZ: {}", e)))?;

		Ok(PyBytes::new(py, &bytes))
	}

	/// Convert coordinates to a different coordinate system.
	///
	/// Args:
	///     from_system: The current coordinate system of the data.
	///     to_system: The target coordinate system.
	fn convert_coordinates(&mut self, from_system: CoordinateSystem, to_system: CoordinateSystem) {
		self.inner
			.convert_coordinates(from_system.inner, to_system.inner);
	}

	/// Rotate 180 degrees about the X axis (RUB <-> RDF conversion).
	fn rotate_180_deg_about_x(&mut self) {
		self.inner.rotate_180_deg_about_x();
	}

	/// The number of Gaussian points.
	#[getter]
	fn num_points(&self) -> i32 {
		self.inner.num_points
	}

	/// The spherical harmonics degree (0-3).
	#[getter]
	fn sh_degree(&self) -> i32 {
		self.inner.spherical_harmonics_degree
	}

	/// Whether the splat was trained with antialiasing.
	#[getter]
	fn antialiased(&self) -> bool {
		self.inner.antialiased
	}

	/// Flattened (x, y, z) positions. Length is num_points * 3.
	#[getter]
	fn positions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
		let list = PyList::new(py, &self.inner.positions)?;
		Ok(list)
	}

	/// Flattened (x, y, z) log-scale values. Length is num_points * 3.
	#[getter]
	fn scales<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
		let list = PyList::new(py, &self.inner.scales)?;
		Ok(list)
	}

	/// Flattened (x, y, z, w) quaternion rotations. Length is num_points * 4.
	#[getter]
	fn rotations<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
		let list = PyList::new(py, &self.inner.rotations)?;
		Ok(list)
	}

	/// Inverse-sigmoid opacity values. Length is num_points.
	#[getter]
	fn alphas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
		let list = PyList::new(py, &self.inner.alphas)?;
		Ok(list)
	}

	/// Flattened (r, g, b) SH0 color values. Length is num_points * 3.
	#[getter]
	fn colors<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
		let list = PyList::new(py, &self.inner.colors)?;
		Ok(list)
	}

	/// Flattened spherical harmonics coefficients.
	#[getter]
	fn spherical_harmonics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
		let list = PyList::new(py, &self.inner.spherical_harmonics)?;
		Ok(list)
	}

	/// Get the bounding box of the splat.
	#[getter]
	fn bbox(&self) -> BoundingBox {
		let bb = self.inner.bbox();
		BoundingBox {
			min_x: bb.min_x,
			max_x: bb.max_x,
			min_y: bb.min_y,
			max_y: bb.max_y,
			min_z: bb.min_z,
			max_z: bb.max_z,
		}
	}

	/// Get the median ellipsoid volume of the Gaussians.
	///
	/// This is useful for understanding the typical size of the
	/// Gaussians in the point cloud.
	#[getter]
	fn median_volume(&self) -> f32 {
		self.inner.median_volume()
	}

	fn __repr__(&self) -> String {
		format!(
			"GaussianSplat(num_points={}, sh_degree={}, antialiased={})",
			self.inner.num_points, self.inner.spherical_harmonics_degree, self.inner.antialiased
		)
	}

	fn __str__(&self) -> String {
		self.inner.to_string()
	}

	fn __len__(&self) -> usize {
		self.inner.num_points as usize
	}
}

/// Load a GaussianSplat from an SPZ file.
///
/// This is a convenience function equivalent to ``GaussianSplat.load()``.
///
/// Args:
///     path: Path to the SPZ file.
///     coordinate_system: Target coordinate system for the loaded data.
///
/// Returns:
///     The loaded Gaussian splat.
#[pyfunction]
#[pyo3(signature = (path, coordinate_system=None))]
fn load(path: &str, coordinate_system: Option<CoordinateSystem>) -> PyResult<GaussianSplat> {
	GaussianSplat::load(path, coordinate_system)
}

/// SPZ - Gaussian Splat file format library.
///
/// A fast, efficient library for reading and writing SPZ files,
/// which store 3D Gaussian Splat point clouds in a compressed format.
///
/// Attributes:
///     GaussianSplat: A 3D Gaussian Splat point cloud.
///     CoordinateSystem: Enumeration of coordinate systems (RUB, RDF, etc.).
///     BoundingBox: Axis-aligned bounding box.
///     load: Load a GaussianSplat from an SPZ file.
///
/// Example:
///     >>> import spz
///     >>> splat = spz.load("scene.spz")
///     >>> print(f"Loaded {len(splat)} gaussians")
///     >>> splat.save("output.spz")
#[pymodule]
fn spz(m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_class::<GaussianSplat>()?;
	m.add_class::<CoordinateSystem>()?;
	m.add_class::<BoundingBox>()?;
	m.add_function(wrap_pyfunction!(load, m)?)?;
	Ok(())
}
