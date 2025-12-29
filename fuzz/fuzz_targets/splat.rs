// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Fuzz target for roundtrip pack/unpack.
//!
//! This tests that packing and unpacking a GaussianSplat produces
//! valid output without panics.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use spz::prelude::{GaussianSplat, PackOptions, UnpackOptions};

#[derive(Debug, Arbitrary)]
struct Input {
	gs: GaussianSplat,
	pack_opts: PackOptions,
	unpack_opts: UnpackOptions,
}

fuzz_target!(|input: Input| {
	if let Ok(bytes) = input.gs.serialize_as_packed_bytes(&input.pack_opts) {
		if let Ok(packed_bytes) = GaussianSplat::load_packed(&bytes) {
			let _ = GaussianSplat::new_from_packed_gaussians(
				&packed_bytes,
				&input.unpack_opts,
			);
		}
	}
});
