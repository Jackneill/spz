// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Fuzz target for coordinate conversion.
//!
//! Tests coordinate system transformations and rotations with arbitrary data.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use spz::coord::CoordinateSystem;

#[derive(Debug, Arbitrary)]
struct Input {
	from_cs: CoordinateSystem,
	to_cs: CoordinateSystem,
}

fuzz_target!(|input: Input| {
	let _ = input.from_cs.axis_flips_to(input.to_cs.clone());
	let _ = input.from_cs.axes_align(input.to_cs);
});
