// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Fuzz target for coordinate conversion.
//!
//! Tests coordinate system transformations and rotations with arbitrary data.

#![no_main]

use libfuzzer_sys::fuzz_target;
use spz::{
	gaussian_splat::{GaussianSplat, LoadOptions},
	packed::PackedGaussians,
};

fuzz_target!(|input: (PackedGaussians, LoadOptions)| {
	let (pg, load_opts) = input;

	let _ = GaussianSplat::new_from_packed_gaussians(&pg.clone(), &load_opts);
});
