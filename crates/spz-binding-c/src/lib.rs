// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SPZ file format handling for C (in Rust).
//!
//! This crate provides a C-compatible API (bindings for the rust crate) for
//! loading and saving Gaussian Splat data in the SPZ format.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, c_char, c_int};
use std::ptr;
use std::slice;

use spz::gaussian_splat::{
	BoundingBox as RustBoundingBox, GaussianSplat as RustGaussianSplat, LoadOptions,
	SaveOptions,
};
use spz::{coord::CoordinateSystem as RustCoordinateSystem, packed::PackedGaussianSplat};

// Thread-local storage for the last error message.
thread_local! {
	static LAST_ERROR: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn set_last_error(msg: String) {
	LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

fn clear_last_error() {
	LAST_ERROR.with(|e| *e.borrow_mut() = None);
}

/// Returns the last error message, or NULL if no error occurred.
///
/// The returned string is valid until the next SPZ function call on this thread.
/// The caller must NOT free this string.
#[unsafe(no_mangle)]
pub extern "C" fn spz_last_error() -> *const c_char {
	LAST_ERROR.with(|e| {
		e.borrow()
			.as_ref()
			.map(|s| s.as_ptr() as *const c_char)
			.unwrap_or(ptr::null())
	})
}

/// Coordinate system enumeration for 3D data.
///
/// The SPZ format internally uses RightUpBack (RUB) coordinates.
/// Specify your source/target coordinate system to enable automatic conversion.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpzCoordinateSystem {
	Unspecified = 0,
	LeftDownBack = 1,
	RightDownBack = 2,
	LeftUpBack = 3,
	RightUpBack = 4,
	LeftDownFront = 5,
	RightDownFront = 6,
	LeftUpFront = 7,
	RightUpFront = 8,
}

impl From<SpzCoordinateSystem> for RustCoordinateSystem {
	fn from(cs: SpzCoordinateSystem) -> Self {
		match cs {
			SpzCoordinateSystem::Unspecified => RustCoordinateSystem::Unspecified,
			SpzCoordinateSystem::LeftDownBack => RustCoordinateSystem::LeftDownBack,
			SpzCoordinateSystem::RightDownBack => RustCoordinateSystem::RightDownBack,
			SpzCoordinateSystem::LeftUpBack => RustCoordinateSystem::LeftUpBack,
			SpzCoordinateSystem::RightUpBack => RustCoordinateSystem::RightUpBack,
			SpzCoordinateSystem::LeftDownFront => RustCoordinateSystem::LeftDownFront,
			SpzCoordinateSystem::RightDownFront => RustCoordinateSystem::RightDownFront,
			SpzCoordinateSystem::LeftUpFront => RustCoordinateSystem::LeftUpFront,
			SpzCoordinateSystem::RightUpFront => RustCoordinateSystem::RightUpFront,
		}
	}
}

impl From<RustCoordinateSystem> for SpzCoordinateSystem {
	fn from(cs: RustCoordinateSystem) -> Self {
		match cs {
			RustCoordinateSystem::Unspecified => SpzCoordinateSystem::Unspecified,
			RustCoordinateSystem::LeftDownBack => SpzCoordinateSystem::LeftDownBack,
			RustCoordinateSystem::RightDownBack => SpzCoordinateSystem::RightDownBack,
			RustCoordinateSystem::LeftUpBack => SpzCoordinateSystem::LeftUpBack,
			RustCoordinateSystem::RightUpBack => SpzCoordinateSystem::RightUpBack,
			RustCoordinateSystem::LeftDownFront => SpzCoordinateSystem::LeftDownFront,
			RustCoordinateSystem::RightDownFront => SpzCoordinateSystem::RightDownFront,
			RustCoordinateSystem::LeftUpFront => SpzCoordinateSystem::LeftUpFront,
			RustCoordinateSystem::RightUpFront => SpzCoordinateSystem::RightUpFront,
		}
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SpzBoundingBox {
	pub min_x: f32,
	pub max_x: f32,
	pub min_y: f32,
	pub max_y: f32,
	pub min_z: f32,
	pub max_z: f32,
}

impl From<RustBoundingBox> for SpzBoundingBox {
	fn from(bbox: RustBoundingBox) -> Self {
		SpzBoundingBox {
			min_x: bbox.min_x,
			max_x: bbox.max_x,
			min_y: bbox.min_y,
			max_y: bbox.max_y,
			min_z: bbox.min_z,
			max_z: bbox.max_z,
		}
	}
}

/// Opaque handle to a GaussianSplat object.
///
/// This handle must be freed with `spz_gaussian_splat_free` when no longer needed.
pub struct SpzGaussianSplat {
	inner: RustGaussianSplat,
}

/// Creates a new empty GaussianSplat.
///
/// Returns NULL on failure. Call `spz_last_error()` for error details.
/// The caller must free the returned handle with `spz_gaussian_splat_free`.
#[unsafe(no_mangle)]
pub extern "C" fn spz_gaussian_splat_new() -> *mut SpzGaussianSplat {
	clear_last_error();
	Box::into_raw(Box::new(SpzGaussianSplat {
		inner: RustGaussianSplat::default(),
	}))
}

/// Loads a GaussianSplat from an SPZ file.
///
/// # Args
///
/// - `filepath`: Path to the SPZ file (null-terminated UTF-8 string).
/// - `coord_sys`: Target coordinate system for the loaded data.
///
/// Returns NULL on failure. Call `spz_last_error()` for error details.
/// The caller must free the returned handle with `spz_gaussian_splat_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_load(
	filepath: *const c_char,
	coord_sys: SpzCoordinateSystem,
) -> *mut SpzGaussianSplat {
	clear_last_error();

	if filepath.is_null() {
		set_last_error("filepath is null".to_string());
		return ptr::null_mut();
	}

	let path = match unsafe { CStr::from_ptr(filepath) }.to_str() {
		Ok(s) => s,
		Err(e) => {
			set_last_error(format!("invalid UTF-8 in filepath: {e}"));
			return ptr::null_mut();
		},
	};

	let opts = LoadOptions {
		coord_sys: coord_sys.into(),
	};

	match RustGaussianSplat::load_with(path, &opts) {
		Ok(gs) => Box::into_raw(Box::new(SpzGaussianSplat { inner: gs })),
		Err(e) => {
			set_last_error(format!("failed to load SPZ file: {e}"));
			ptr::null_mut()
		},
	}
}

/// Loads a GaussianSplat from a byte buffer containing SPZ data.
///
/// # Args
///
/// - `data`: Pointer to the SPZ data buffer.
/// - `len`: Length of the data buffer in bytes.
/// - `coord_sys`: Target coordinate system for the loaded data.
///
/// Returns NULL on failure. Call `spz_last_error()` for error details.
/// The caller must free the returned handle with `spz_gaussian_splat_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_load_from_bytes(
	data: *const u8,
	len: usize,
	coord_sys: SpzCoordinateSystem,
) -> *mut SpzGaussianSplat {
	clear_last_error();

	if data.is_null() {
		set_last_error("data pointer is null".to_string());
		return ptr::null_mut();
	}

	let bytes = unsafe { slice::from_raw_parts(data, len) };

	let opts = LoadOptions {
		coord_sys: coord_sys.into(),
	};
	match PackedGaussianSplat::from_bytes(bytes) {
		Ok(packed) => match RustGaussianSplat::new_from_packed_gaussians(&packed, &opts) {
			Ok(gs) => Box::into_raw(Box::new(SpzGaussianSplat { inner: gs })),
			Err(e) => {
				set_last_error(format!("failed to unpack SPZ data: {e}"));
				ptr::null_mut()
			},
		},
		Err(e) => {
			set_last_error(format!("failed to decompress SPZ data: {e}"));
			ptr::null_mut()
		},
	}
}

/// Saves a GaussianSplat to an SPZ file.
///
/// # Args
///
/// - `splat`: Handle to the GaussianSplat.
/// - `filepath`: Path to save the SPZ file (null-terminated UTF-8 string).
/// - `coord_sys`: Source coordinate system of the data being saved.
///
/// Returns 0 on success, -1 on failure. Call `spz_last_error()` for error details.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_save(
	splat: *const SpzGaussianSplat,
	filepath: *const c_char,
	coord_sys: SpzCoordinateSystem,
) -> c_int {
	clear_last_error();

	if splat.is_null() {
		set_last_error("splat handle is null".to_string());
		return -1;
	}

	if filepath.is_null() {
		set_last_error("filepath is null".to_string());
		return -1;
	}

	let splat = unsafe { &*splat };
	let path = match unsafe { CStr::from_ptr(filepath) }.to_str() {
		Ok(s) => s,
		Err(e) => {
			set_last_error(format!("invalid UTF-8 in filepath: {e}"));
			return -1;
		},
	};

	let opts = SaveOptions {
		coord_sys: coord_sys.into(),
	};

	match splat.inner.save(path, &opts) {
		Ok(()) => 0,
		Err(e) => {
			set_last_error(format!("failed to save SPZ file: {e}"));
			-1
		},
	}
}

/// Serializes a GaussianSplat to a byte buffer.
///
/// # Args
///
/// - `splat`: Handle to the GaussianSplat.
/// - `coord_sys`: Source coordinate system of the data being saved.
/// - `out_data`: Pointer to receive the allocated data buffer.
/// - `out_len`: Pointer to receive the buffer length.
///
/// Returns 0 on success, -1 on failure. Call `spz_last_error()` for error details.
/// The caller must free the returned buffer with `spz_free_bytes`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_to_bytes(
	splat: *const SpzGaussianSplat,
	coord_sys: SpzCoordinateSystem,
	out_data: *mut *mut u8,
	out_len: *mut usize,
) -> c_int {
	clear_last_error();

	if splat.is_null() || out_data.is_null() || out_len.is_null() {
		set_last_error("null pointer argument".to_string());
		return -1;
	}

	let splat = unsafe { &*splat };
	let opts = SaveOptions {
		coord_sys: coord_sys.into(),
	};

	match splat.inner.serialize_to_packed_bytes(&opts) {
		Ok(bytes) => {
			let len = bytes.len();
			let boxed = bytes.into_boxed_slice();
			let ptr = Box::into_raw(boxed) as *mut u8;
			unsafe {
				*out_data = ptr;
				*out_len = len;
			}
			0
		},
		Err(e) => {
			set_last_error(format!("failed to serialize SPZ data: {e}"));
			-1
		},
	}
}

/// Frees a byte buffer allocated by `spz_gaussian_splat_to_bytes`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_free_bytes(data: *mut u8, len: usize) {
	if !data.is_null() {
		let _ = unsafe { Box::from_raw(slice::from_raw_parts_mut(data, len)) };
	}
}

/// Frees a GaussianSplat handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_free(splat: *mut SpzGaussianSplat) {
	if !splat.is_null() {
		let _ = unsafe { Box::from_raw(splat) };
	}
}

/// Returns the number of points (gaussians) in the splat.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_num_points(splat: *const SpzGaussianSplat) -> i32 {
	if splat.is_null() {
		return 0;
	}
	unsafe { &*splat }.inner.header.num_points
}

/// Returns the spherical harmonics degree (0-3).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_sh_degree(splat: *const SpzGaussianSplat) -> u8 {
	if splat.is_null() {
		return 0;
	}
	unsafe { &*splat }.inner.header.spherical_harmonics_degree
}

/// Returns whether the splat was trained with antialiasing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_antialiased(splat: *const SpzGaussianSplat) -> bool {
	if splat.is_null() {
		return false;
	}
	unsafe { &*splat }.inner.header.flags.is_antialiased()
}

/// Returns the bounding box of the splat.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_bbox(splat: *const SpzGaussianSplat) -> SpzBoundingBox {
	if splat.is_null() {
		return SpzBoundingBox {
			min_x: 0.0,
			max_x: 0.0,
			min_y: 0.0,
			max_y: 0.0,
			min_z: 0.0,
			max_z: 0.0,
		};
	}
	unsafe { &*splat }.inner.bbox().into()
}

/// Returns the median ellipsoid volume of the gaussians.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_median_volume(splat: *const SpzGaussianSplat) -> f32 {
	if splat.is_null() {
		return 0.0;
	}
	unsafe { &*splat }.inner.median_volume()
}

/// Returns a pointer to the positions array.
///
/// The array contains `num_points * 3` floats in [x0, y0, z0, x1, y1, z1, ...] order.
/// The pointer is valid until the splat is modified or freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_positions(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	if splat.is_null() {
		if !out_len.is_null() {
			unsafe { *out_len = 0 };
		}
		return ptr::null();
	}
	let splat = unsafe { &*splat };
	if !out_len.is_null() {
		unsafe { *out_len = splat.inner.positions.len() };
	}
	splat.inner.positions.as_ptr()
}

/// Returns a pointer to the scales array.
///
/// The array contains `num_points * 3` floats (log-encoded) in [x0, y0, z0, ...] order.
/// The pointer is valid until the splat is modified or freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_scales(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	if splat.is_null() {
		if !out_len.is_null() {
			unsafe { *out_len = 0 };
		}
		return ptr::null();
	}
	let splat = unsafe { &*splat };
	if !out_len.is_null() {
		unsafe { *out_len = splat.inner.scales.len() };
	}
	splat.inner.scales.as_ptr()
}

/// Returns a pointer to the rotations array.
///
/// The array contains `num_points * 4` floats (quaternions) in [x0, y0, z0, w0, ...] order.
/// The pointer is valid until the splat is modified or freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_rotations(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	if splat.is_null() {
		if !out_len.is_null() {
			unsafe { *out_len = 0 };
		}
		return ptr::null();
	}
	let splat = unsafe { &*splat };
	if !out_len.is_null() {
		unsafe { *out_len = splat.inner.rotations.len() };
	}
	splat.inner.rotations.as_ptr()
}

/// Returns a pointer to the alphas (opacity) array.
///
/// The array contains `num_points` floats (sigmoid-encoded opacity values).
/// The pointer is valid until the splat is modified or freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_alphas(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	if splat.is_null() {
		if !out_len.is_null() {
			unsafe { *out_len = 0 };
		}
		return ptr::null();
	}
	let splat = unsafe { &*splat };
	if !out_len.is_null() {
		unsafe { *out_len = splat.inner.alphas.len() };
	}
	splat.inner.alphas.as_ptr()
}

/// Returns a pointer to the colors array.
///
/// The array contains `num_points * 3` floats (DC color) in [r0, g0, b0, ...] order.
/// The pointer is valid until the splat is modified or freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_colors(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	if splat.is_null() {
		if !out_len.is_null() {
			unsafe { *out_len = 0 };
		}
		return ptr::null();
	}
	let splat = unsafe { &*splat };
	if !out_len.is_null() {
		unsafe { *out_len = splat.inner.colors.len() };
	}
	splat.inner.colors.as_ptr()
}

/// Returns a pointer to the spherical harmonics array.
///
/// The array contains SH coefficients for degrees 1-3.
/// The number of coefficients per gaussian depends on the SH degree:
/// - Degree 0: 0 coefficients
/// - Degree 1: 9 coefficients (3 bands × 3 colors)
/// - Degree 2: 24 coefficients
/// - Degree 3: 45 coefficients
///
/// The pointer is valid until the splat is modified or freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_spherical_harmonics(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	if splat.is_null() {
		if !out_len.is_null() {
			unsafe { *out_len = 0 };
		}
		return ptr::null();
	}
	let splat = unsafe { &*splat };
	if !out_len.is_null() {
		unsafe { *out_len = splat.inner.spherical_harmonics.len() };
	}
	splat.inner.spherical_harmonics.as_ptr()
}

/// Converts the splat's coordinate system in-place.
///
/// # Args
///
/// - `splat`: Handle to the GaussianSplat.
/// - `from`: Source coordinate system.
/// - `to`: Target coordinate system.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_convert_coordinates(
	splat: *mut SpzGaussianSplat,
	from: SpzCoordinateSystem,
	to: SpzCoordinateSystem,
) {
	if splat.is_null() {
		return;
	}
	let splat = unsafe { &mut *splat };
	splat.inner.convert_coordinates(from.into(), to.into());
}

/// Returns the library version as a static string.
#[unsafe(no_mangle)]
pub extern "C" fn spz_version() -> *const c_char {
	concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}
