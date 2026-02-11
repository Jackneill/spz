// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Fuzz target for coordinate conversion.
//!
//! Tests coordinate system transformations and rotations with arbitrary data.

#![no_main]

use libfuzzer_sys::fuzz_target;
use spz::header::Header;

fuzz_target!(|data: &[u8]| {
	let Ok(header_bytes) = <[u8; 16]>::try_from(data) else {
		return;
	};
	let _ = Header::try_from(header_bytes);
});
