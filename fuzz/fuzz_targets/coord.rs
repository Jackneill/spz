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
	from_sys: CoordinateSystem,
	to_sys: CoordinateSystem,
}

fuzz_target!(|input: Input| {
	let _ = input.from_sys.axis_flips_to(input.to_sys.clone());
	let _ = input.from_sys.axes_align(input.to_sys);
});
