// SPDX-License-Identifier: Apache-2.0 OR MIT

//! SPZ file format handling for C (in Rust).
//!
//! This crate provides a C-compatible API (bindings for the rust crate) for
//! loading and saving Gaussian Splat data in the SPZ format.
//!
//! # Error handling
//!
//! Functions that can fail return [`SpzResult`]. On failure, call
//! [`spz_last_error`] to retrieve a human-readable, thread-local error message.
//!
//! # Memory management
//!
//! Every `spz_*_new` / `spz_*_load*` function returns a heap-allocated handle
//! that must be freed with the matching `spz_*_free` function. Byte buffers
//! returned via out-parameters must be freed with [`spz_free_bytes`].
//! Strings returned by `spz_*_pretty_fmt` must be freed with
//! [`spz_free_string`].

#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::{CStr, c_char};
use std::ptr;
use std::slice;

use spz::coord::CoordinateSystem as RustCoordinateSystem;
use spz::gaussian_splat::{
	BoundingBox as RustBoundingBox, GaussianSplat as RustGaussianSplat, LoadOptions,
	SaveOptions,
};
use spz::header::{Header as RustHeader, Version as RustVersion};
use spz::packed::PackedGaussianSplat;

// ---------------------------------------------------------------------------
// Thread-local error handling
// ---------------------------------------------------------------------------

thread_local! {
static LAST_ERROR: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn set_last_error(msg: String) {
	LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

fn clear_last_error() {
	LAST_ERROR.with(|e| *e.borrow_mut() = None);
}

fn cstr_arg<'a>(ptr: *const c_char, name: &str) -> std::result::Result<&'a str, String> {
	if ptr.is_null() {
		return Err(format!("{name} is null"));
	}

	// SAFETY: `ptr` is checked for null above, and the FFI contract for callers
	// requires a valid, NUL-terminated C string for the duration of the call.
	unsafe { CStr::from_ptr(ptr) }
		.to_str()
		.map_err(|e| format!("invalid UTF-8 in {name}: {e}"))
}

fn byte_slice_arg<'a>(data: *const u8, len: usize) -> std::result::Result<&'a [u8], String> {
	if data.is_null() {
		return Err("data pointer is null".to_string());
	}

	// SAFETY: `data` is checked for null above, and the FFI contract for callers
	// requires that it points to `len` readable bytes for the duration of the call.
	Ok(unsafe { slice::from_raw_parts(data, len) })
}

fn header_ref(header: *const SpzHeader) -> Option<&'static SpzHeader> {
	if header.is_null() {
		return None;
	}

	// SAFETY: The public FFI API documents that non-null header handles must be
	// live pointers previously returned by this library.
	Some(unsafe { &*header })
}

fn splat_ref(splat: *const SpzGaussianSplat) -> Option<&'static SpzGaussianSplat> {
	if splat.is_null() {
		return None;
	}

	// SAFETY: The public FFI API documents that non-null splat handles must be
	// live pointers previously returned by this library.
	Some(unsafe { &*splat })
}

fn splat_mut(splat: *mut SpzGaussianSplat) -> Option<&'static mut SpzGaussianSplat> {
	if splat.is_null() {
		return None;
	}

	// SAFETY: The public FFI API documents that non-null mutable splat handles
	// must be unique, live pointers previously returned by this library.
	Some(unsafe { &mut *splat })
}

fn write_out_len(out_len: *mut usize, len: usize) {
	if out_len.is_null() {
		return;
	}

	// SAFETY: When provided, `out_len` must be a valid writable pointer for this call.
	unsafe { *out_len = len };
}

fn free_box_handle<T>(ptr: *mut T) {
	if ptr.is_null() {
		return;
	}

	// SAFETY: The matching free functions are only valid for heap pointers
	// allocated and returned by this library.
	let _ = unsafe { Box::from_raw(ptr) };
}

fn free_byte_buffer(data: *mut u8, len: usize) {
	if data.is_null() {
		return;
	}

	// SAFETY: `data` and `len` must come from `spz_gaussian_splat_to_bytes`,
	// which allocates this exact boxed slice layout.
	let _ = unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(data, len)) };
}

fn free_c_string(s: *mut c_char) {
	if s.is_null() {
		return;
	}

	// SAFETY: `s` must come from one of this library's string-returning APIs,
	// all of which allocate with `CString::into_raw`.
	let _ = unsafe { std::ffi::CString::from_raw(s) };
}

// ---------------------------------------------------------------------------
// Result type
// ---------------------------------------------------------------------------

/// Status code returned by fallible SPZ functions.
///
/// Check with `result == SpzResult_Success`. On failure, call
/// `spz_last_error()` for a descriptive message.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpzResult {
	/// Operation completed successfully.
	Success = 0,
	/// A null pointer was passed where a valid pointer was expected.
	NullPointer = 1,
	/// A function argument was invalid (e.g. non-UTF-8 path).
	InvalidArgument = 2,
	/// An I/O or parsing error occurred.
	IoError = 3,
}

// ---------------------------------------------------------------------------
// Coordinate system
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

/// SPZ file format version.
///
/// Currently only V2 and V3 are supported. V3 is the default and recommended
/// version.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpzVersion {
	/// Version 1 (unsupported).
	V1 = 1,
	/// Version 2.
	V2 = 2,
	/// Version 3 (default).
	V3 = 3,
}

impl From<RustVersion> for SpzVersion {
	fn from(v: RustVersion) -> Self {
		match v {
			RustVersion::V1 => SpzVersion::V1,
			RustVersion::V2 => SpzVersion::V2,
			RustVersion::V3 => SpzVersion::V3,
		}
	}
}

impl From<SpzVersion> for RustVersion {
	fn from(v: SpzVersion) -> Self {
		match v {
			SpzVersion::V1 => RustVersion::V1,
			SpzVersion::V2 => RustVersion::V2,
			SpzVersion::V3 => RustVersion::V3,
		}
	}
}

// ---------------------------------------------------------------------------
// Bounding box
// ---------------------------------------------------------------------------

/// Axis-aligned bounding box of a Gaussian Splat.
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

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

/// Opaque handle to an SPZ file header.
///
/// A header can be read from a file or from bytes *without* loading the full
/// splat data. This is useful for quick file inspection.
///
/// Must be freed with `spz_header_free`.
pub struct SpzHeader {
	inner: RustHeader,
}

/// Reads a header from an SPZ file without loading the full splat data.
///
/// Efficient for quickly inspecting SPZ file metadata.
///
/// Returns NULL on failure. Call `spz_last_error()` for error details.
/// The caller must free the returned handle with `spz_header_free`.
///
/// # Safety
///
/// `filepath` must be a valid, non-null pointer to a NUL-terminated string
/// for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_from_file(filepath: *const c_char) -> *mut SpzHeader {
	clear_last_error();

	let path = match cstr_arg(filepath, "filepath") {
		Ok(path) => path,
		Err(message) => {
			set_last_error(message);
			return ptr::null_mut();
		},
	};

	match RustHeader::from_file(path) {
		Ok(h) => Box::into_raw(Box::new(SpzHeader { inner: h })),
		Err(e) => {
			set_last_error(format!("failed to read SPZ header: {e}"));
			ptr::null_mut()
		},
	}
}

/// Reads a header from compressed SPZ bytes without loading the full splat data.
///
/// Returns NULL on failure. Call `spz_last_error()` for error details.
/// The caller must free the returned handle with `spz_header_free`.
///
/// # Safety
///
/// `data` must be a valid, non-null pointer to `len` readable bytes for the
/// duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_from_bytes(data: *const u8, len: usize) -> *mut SpzHeader {
	clear_last_error();

	let bytes = match byte_slice_arg(data, len) {
		Ok(bytes) => bytes,
		Err(message) => {
			set_last_error(message);
			return ptr::null_mut();
		},
	};

	match RustHeader::from_compressed_bytes(bytes) {
		Ok(h) => Box::into_raw(Box::new(SpzHeader { inner: h })),
		Err(e) => {
			set_last_error(format!("failed to parse SPZ header: {e}"));
			ptr::null_mut()
		},
	}
}

/// Frees a header handle.
///
/// # Safety
///
/// `header` must be null or a pointer previously returned by this library and
/// not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_free(header: *mut SpzHeader) {
	free_box_handle(header);
}

/// Returns the SPZ format version stored in the header.
///
/// # Safety
///
/// `header` must be null or a valid live header handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_version(header: *const SpzHeader) -> SpzVersion {
	header_ref(header)
		.map(|header| header.inner.version.into())
		.unwrap_or(SpzVersion::V3)
}

/// Returns the number of Gaussian points recorded in the header.
///
/// # Safety
///
/// `header` must be null or a valid live header handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_num_points(header: *const SpzHeader) -> i32 {
	header_ref(header)
		.map(|header| header.inner.num_points)
		.unwrap_or(0)
}

/// Returns the spherical harmonics degree (0-3).
///
/// # Safety
///
/// `header` must be null or a valid live header handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_sh_degree(header: *const SpzHeader) -> u8 {
	header_ref(header)
		.map(|header| header.inner.spherical_harmonics_degree)
		.unwrap_or(0)
}

/// Returns the number of fractional bits used in position encoding.
///
/// Standard value is 12, giving ~0.25 mm resolution.
///
/// # Safety
///
/// `header` must be null or a valid live header handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_fractional_bits(header: *const SpzHeader) -> u8 {
	header_ref(header)
		.map(|header| header.inner.fractional_bits)
		.unwrap_or(0)
}

/// Returns whether the splat was trained with antialiasing.
///
/// # Safety
///
/// `header` must be null or a valid live header handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_antialiased(header: *const SpzHeader) -> bool {
	header_ref(header)
		.map(|header| header.inner.flags.is_antialiased())
		.unwrap_or(false)
}

/// Validates the header (magic number, version, ranges, reserved bytes).
///
/// Returns `true` if the header passes all validation checks.
///
/// # Safety
///
/// `header` must be null or a valid live header handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_is_valid(header: *const SpzHeader) -> bool {
	header_ref(header)
		.map(|header| header.inner.is_valid())
		.unwrap_or(false)
}

/// Returns a heap-allocated, human-readable summary of the header.
///
/// The caller must free the returned string with `spz_free_string`.
/// Returns NULL if the handle is null.
///
/// # Safety
///
/// `header` must be null or a valid live header handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_header_pretty_fmt(header: *const SpzHeader) -> *mut c_char {
	let Some(header) = header_ref(header) else {
		return ptr::null_mut();
	};
	let s = header.inner.pretty_fmt();

	match std::ffi::CString::new(s) {
		Ok(cs) => cs.into_raw(),
		Err(_) => ptr::null_mut(),
	}
}

// ---------------------------------------------------------------------------
// GaussianSplat
// ---------------------------------------------------------------------------

/// Opaque handle to a GaussianSplat object.
///
/// Must be freed with `spz_gaussian_splat_free`.
pub struct SpzGaussianSplat {
	inner: RustGaussianSplat,
}

/// Creates a new, empty GaussianSplat (zero points).
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
/// Returns NULL on failure. Call `spz_last_error()` for error details.
/// The caller must free the returned handle with `spz_gaussian_splat_free`.
///
/// # Safety
///
/// `filepath` must be a valid, non-null pointer to a NUL-terminated string
/// for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_load(
	filepath: *const c_char,
	coord_sys: SpzCoordinateSystem,
) -> *mut SpzGaussianSplat {
	clear_last_error();

	let path = match cstr_arg(filepath, "filepath") {
		Ok(path) => path,
		Err(message) => {
			set_last_error(message);
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
/// Returns NULL on failure. Call `spz_last_error()` for error details.
/// The caller must free the returned handle with `spz_gaussian_splat_free`.
///
/// # Safety
///
/// `data` must be a valid, non-null pointer to `len` readable bytes for the
/// duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_load_from_bytes(
	data: *const u8,
	len: usize,
	coord_sys: SpzCoordinateSystem,
) -> *mut SpzGaussianSplat {
	clear_last_error();

	let bytes = match byte_slice_arg(data, len) {
		Ok(bytes) => bytes,
		Err(message) => {
			set_last_error(message);
			return ptr::null_mut();
		},
	};

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
/// Returns `SpzResult_Success` on success. Call `spz_last_error()` on failure.
///
/// # Safety
///
/// `splat` must be a valid live handle returned by this library, and `filepath`
/// must be a valid, non-null pointer to a NUL-terminated string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_save(
	splat: *const SpzGaussianSplat,
	filepath: *const c_char,
	coord_sys: SpzCoordinateSystem,
) -> SpzResult {
	clear_last_error();

	let Some(splat) = splat_ref(splat) else {
		set_last_error("splat handle is null".to_string());
		return SpzResult::NullPointer;
	};
	let path = match cstr_arg(filepath, "filepath") {
		Ok(path) => path,
		Err(message) => {
			set_last_error(message);
			return SpzResult::InvalidArgument;
		},
	};

	let opts = SaveOptions {
		coord_sys: coord_sys.into(),
	};

	match splat.inner.save(path, &opts) {
		Ok(()) => SpzResult::Success,
		Err(e) => {
			set_last_error(format!("failed to save SPZ file: {e}"));
			SpzResult::IoError
		},
	}
}

/// Serializes a GaussianSplat to a heap-allocated byte buffer.
///
/// Returns `SpzResult_Success` on success. Call `spz_last_error()` on failure.
/// The caller must free the returned buffer with `spz_free_bytes`.
///
/// # Safety
///
/// `splat` must be a valid live handle returned by this library. `out_data`
/// and `out_len` must be valid writable pointers for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_to_bytes(
	splat: *const SpzGaussianSplat,
	coord_sys: SpzCoordinateSystem,
	out_data: *mut *mut u8,
	out_len: *mut usize,
) -> SpzResult {
	clear_last_error();

	let Some(splat) = splat_ref(splat) else {
		set_last_error("null pointer argument".to_string());
		return SpzResult::NullPointer;
	};
	if out_data.is_null() || out_len.is_null() {
		set_last_error("null pointer argument".to_string());
		return SpzResult::NullPointer;
	}
	let opts = SaveOptions {
		coord_sys: coord_sys.into(),
	};

	match splat.inner.serialize_to_packed_bytes(&opts) {
		Ok(bytes) => {
			let len = bytes.len();
			let boxed = bytes.into_boxed_slice();
			let ptr = Box::into_raw(boxed) as *mut u8;

			// SAFETY: `out_data` and `out_len` were checked for null above and the
			// FFI contract requires them to be valid writable pointers for this call.
			unsafe {
				*out_data = ptr;
				*out_len = len;
			}
			SpzResult::Success
		},
		Err(e) => {
			set_last_error(format!("failed to serialize SPZ data: {e}"));
			SpzResult::IoError
		},
	}
}

/// Frees a byte buffer previously returned by `spz_gaussian_splat_to_bytes`.
///
/// # Safety
///
/// `data` and `len` must match a buffer previously returned by
/// `spz_gaussian_splat_to_bytes` and not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_free_bytes(data: *mut u8, len: usize) {
	free_byte_buffer(data, len);
}

/// # Safety
///
/// `splat` must be null or a pointer previously returned by this library and
/// not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_free(splat: *mut SpzGaussianSplat) {
	free_box_handle(splat);
}

// ---------------------------------------------------------------------------
// GaussianSplat — scalar accessors
// ---------------------------------------------------------------------------

/// Returns the number of points (gaussians) in the splat.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_num_points(splat: *const SpzGaussianSplat) -> i32 {
	splat_ref(splat)
		.map(|splat| splat.inner.header.num_points)
		.unwrap_or(0)
}

/// Returns the spherical harmonics degree (0-3).
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_sh_degree(splat: *const SpzGaussianSplat) -> u8 {
	splat_ref(splat)
		.map(|splat| splat.inner.header.spherical_harmonics_degree)
		.unwrap_or(0)
}

/// Returns the SPZ format version of the splat.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_version(splat: *const SpzGaussianSplat) -> SpzVersion {
	splat_ref(splat)
		.map(|splat| splat.inner.header.version.into())
		.unwrap_or(SpzVersion::V3)
}

/// Returns the number of fractional bits used in position encoding.
///
/// Standard value is 12, giving ~0.25 mm resolution.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_fractional_bits(splat: *const SpzGaussianSplat) -> u8 {
	splat_ref(splat)
		.map(|splat| splat.inner.header.fractional_bits)
		.unwrap_or(0)
}

/// Returns whether the splat was trained with antialiasing.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_antialiased(splat: *const SpzGaussianSplat) -> bool {
	splat_ref(splat)
		.map(|splat| splat.inner.header.flags.is_antialiased())
		.unwrap_or(false)
}

/// Returns the bounding box of the splat.
///
/// Returns a zeroed bounding box if the handle is null.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_bbox(splat: *const SpzGaussianSplat) -> SpzBoundingBox {
	let Some(splat) = splat_ref(splat) else {
		return SpzBoundingBox {
			min_x: 0.0,
			max_x: 0.0,
			min_y: 0.0,
			max_y: 0.0,
			min_z: 0.0,
			max_z: 0.0,
		};
	};
	splat.inner.bbox().into()
}

/// Returns the median ellipsoid volume of the gaussians.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_median_volume(splat: *const SpzGaussianSplat) -> f32 {
	splat_ref(splat)
		.map(|splat| splat.inner.median_volume())
		.unwrap_or(0.0)
}

/// Validates that all internal arrays have consistent sizes.
///
/// Returns `true` if the splat passes all size checks.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_check_sizes(splat: *const SpzGaussianSplat) -> bool {
	splat_ref(splat)
		.map(|splat| splat.inner.check_sizes())
		.unwrap_or(false)
}

// ---------------------------------------------------------------------------
// GaussianSplat — array accessors
// ---------------------------------------------------------------------------

/// Returns a pointer to the positions array.
///
/// The array contains `num_points * 3` floats in `[x0, y0, z0, x1, y1, z1, ...]` order.
/// The pointer is valid until the splat is modified or freed.
///
/// If `out_len` is non-null it receives the total number of floats.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
/// If `out_len` is non-null it must be a valid writable pointer for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_positions(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	let Some(splat) = splat_ref(splat) else {
		write_out_len(out_len, 0);
		return ptr::null();
	};
	write_out_len(out_len, splat.inner.positions.len());
	splat.inner.positions.as_ptr()
}

/// Returns a pointer to the scales array.
///
/// The array contains `num_points * 3` floats (log-encoded) in `[x0, y0, z0, ...]` order.
/// The pointer is valid until the splat is modified or freed.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
/// If `out_len` is non-null it must be a valid writable pointer for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_scales(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	let Some(splat) = splat_ref(splat) else {
		write_out_len(out_len, 0);
		return ptr::null();
	};
	write_out_len(out_len, splat.inner.scales.len());
	splat.inner.scales.as_ptr()
}

/// Returns a pointer to the rotations array.
///
/// The array contains `num_points * 4` floats (quaternions) in
/// `[x0, y0, z0, w0, ...]` order.
/// The pointer is valid until the splat is modified or freed.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
/// If `out_len` is non-null it must be a valid writable pointer for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_rotations(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	let Some(splat) = splat_ref(splat) else {
		write_out_len(out_len, 0);
		return ptr::null();
	};
	write_out_len(out_len, splat.inner.rotations.len());
	splat.inner.rotations.as_ptr()
}

/// Returns a pointer to the alphas (opacity) array.
///
/// The array contains `num_points` floats (sigmoid-encoded opacity values).
/// The pointer is valid until the splat is modified or freed.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
/// If `out_len` is non-null it must be a valid writable pointer for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_alphas(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	let Some(splat) = splat_ref(splat) else {
		write_out_len(out_len, 0);
		return ptr::null();
	};
	write_out_len(out_len, splat.inner.alphas.len());
	splat.inner.alphas.as_ptr()
}

/// Returns a pointer to the colors array.
///
/// The array contains `num_points * 3` floats (DC colour) in `[r0, g0, b0, ...]` order.
/// The pointer is valid until the splat is modified or freed.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
/// If `out_len` is non-null it must be a valid writable pointer for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_colors(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	let Some(splat) = splat_ref(splat) else {
		write_out_len(out_len, 0);
		return ptr::null();
	};
	write_out_len(out_len, splat.inner.colors.len());
	splat.inner.colors.as_ptr()
}

/// Returns a pointer to the spherical harmonics coefficients array.
///
/// The number of coefficients per gaussian depends on the SH degree:
/// - Degree 0: 0 coefficients
/// - Degree 1: 9 coefficients (3 bands x 3 colours)
/// - Degree 2: 24 coefficients
/// - Degree 3: 45 coefficients
///
/// The pointer is valid until the splat is modified or freed.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
/// If `out_len` is non-null it must be a valid writable pointer for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_spherical_harmonics(
	splat: *const SpzGaussianSplat,
	out_len: *mut usize,
) -> *const f32 {
	let Some(splat) = splat_ref(splat) else {
		write_out_len(out_len, 0);
		return ptr::null();
	};
	write_out_len(out_len, splat.inner.spherical_harmonics.len());
	splat.inner.spherical_harmonics.as_ptr()
}

// ---------------------------------------------------------------------------
// GaussianSplat — mutation
// ---------------------------------------------------------------------------

/// Converts the splat's coordinate system in-place.
///
/// # Safety
///
/// `splat` must be null or a unique live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_convert_coordinates(
	splat: *mut SpzGaussianSplat,
	from: SpzCoordinateSystem,
	to: SpzCoordinateSystem,
) {
	let Some(splat) = splat_mut(splat) else {
		return;
	};

	splat.inner.convert_coordinates(from.into(), to.into());
}

// ---------------------------------------------------------------------------
// GaussianSplat — string helpers
// ---------------------------------------------------------------------------

/// Returns a heap-allocated, human-readable summary of the splat.
///
/// Includes header information, median volume, and bounding box.
/// The caller must free the returned string with `spz_free_string`.
/// Returns NULL if the handle is null.
///
/// # Safety
///
/// `splat` must be null or a valid live splat handle returned by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_gaussian_splat_pretty_fmt(
	splat: *const SpzGaussianSplat,
) -> *mut c_char {
	let Some(splat) = splat_ref(splat) else {
		return ptr::null_mut();
	};
	let s = splat.inner.pretty_fmt();

	match std::ffi::CString::new(s) {
		Ok(cs) => cs.into_raw(),
		Err(_) => ptr::null_mut(),
	}
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

/// Frees a string previously returned by `spz_gaussian_splat_pretty_fmt`
/// or `spz_header_pretty_fmt`.
///
/// # Safety
///
/// `s` must be null or a pointer previously returned by this library and not
/// already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spz_free_string(s: *mut c_char) {
	free_c_string(s);
}

// ---------------------------------------------------------------------------
// Global helpers
// ---------------------------------------------------------------------------

/// Returns the last error message, or NULL if no error has occurred.
///
/// The returned string is valid until the next SPZ function call on the same
/// thread. The caller must NOT free this string.
#[unsafe(no_mangle)]
pub extern "C" fn spz_last_error() -> *const c_char {
	LAST_ERROR.with(|e| {
		e.borrow()
			.as_ref()
			.map(|s| s.as_ptr() as *const c_char)
			.unwrap_or(ptr::null())
	})
}

/// Returns the library version as a static null-terminated string.
#[unsafe(no_mangle)]
pub extern "C" fn spz_version() -> *const c_char {
	concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}
