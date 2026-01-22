// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Fuzz target for roundtrip pack/unpack.
//!
//! This tests that packing and unpacking a GaussianSplat produces
//! valid output without panics.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use spz::{
	packed::PackedGaussianSplat,
	prelude::{GaussianSplat, LoadOptions, SaveOptions},
};

#[derive(Debug, Arbitrary)]
struct Input {
	gs: GaussianSplat,
	save_opts: SaveOptions,
	load_opts: LoadOptions,
}

fuzz_target!(|input: Input| {
	if let Ok(bytes) = input.gs.serialize_to_packed_bytes(&input.save_opts) {
		if let Ok(packed_bytes) = PackedGaussianSplat::from_bytes(&bytes) {
			let _ = GaussianSplat::new_from_packed_gaussians(
				&packed_bytes,
				&input.load_opts,
			);
		}
	}
});
