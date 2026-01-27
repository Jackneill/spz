// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Fuzz target for gzip decompression.
//!
//! Tests that the compression module handles malformed gzip data gracefully.

#![no_main]

use libfuzzer_sys::fuzz_target;
use spz::compression;

fuzz_target!(|data: &[u8]| {
	let mut output = Vec::new();
	let _ = compression::gzip::decompress_end(data, &mut output);
});
