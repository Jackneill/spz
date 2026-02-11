// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::f32::consts::FRAC_1_SQRT_2;

#[inline]
pub fn degree_for_dim(dim: u8) -> u8 {
	if dim < 3 {
		0
	} else if dim < 8 {
		1
	} else if dim < 15 {
		2
	} else {
		3
	}
}

#[inline]
pub fn dim_for_degree(degree: u8) -> u8 {
	match degree {
		0 => 0,
		1 => 3,
		2 => 8,
		3 => 15,
		_ => 0,
	}
}

#[inline]
pub fn unpack_quaternion_first_three(rotation: &mut [f32], r: &[u8]) {
	unpack_quaternion_first_three_with_flip(rotation, r, [1.0_f32, 1.0_f32, 1.0_f32]);
}

pub fn unpack_quaternion_first_three_with_flip(rotation: &mut [f32], r: &[u8], flip_q: [f32; 3]) {
	debug_assert!(rotation.len() >= 4 && r.len() >= 3);

	let scale = 1.0_f32 / 127.5_f32;

	let mut xyz = [
		(r[0] as f32) * scale - 1.0_f32,
		(r[1] as f32) * scale - 1.0_f32,
		(r[2] as f32) * scale - 1.0_f32,
	];
	xyz[0] *= flip_q[0];
	xyz[1] *= flip_q[1];
	xyz[2] *= flip_q[2];

	rotation[0] = xyz[0];
	rotation[1] = xyz[1];
	rotation[2] = xyz[2];

	let sq = xyz[0] * xyz[0] + xyz[1] * xyz[1] + xyz[2] * xyz[2];

	rotation[3] = (1.0_f32 - sq).max(0.0_f32).sqrt();
}

#[inline]
pub fn unpack_quaternion_smallest_three(rotation: &mut [f32], r: &[u8]) {
	unpack_quaternion_smallest_three_with_flip(rotation, r, [1.0_f32, 1.0_f32, 1.0_f32]);
}

pub fn unpack_quaternion_smallest_three_with_flip(
	rotation: &mut [f32],
	r: &[u8],
	flip_q: [f32; 3],
) {
	debug_assert!(rotation.len() >= 4 && r.len() >= 4);

	let mut comp: u32 = (r[0] as u32)
		| ((r[1] as u32) << 8)
		| ((r[2] as u32) << 16)
		| ((r[3] as u32) << 24);

	const C_MASK: u32 = (1_u32 << 9) - 1_u32;

	let i_largest = (comp >> 30) as usize;
	let mut sum_squares: f32 = 0.0;

	for i in (0..4).rev() {
		if i == i_largest {
			continue;
		}
		let mag = comp & C_MASK;
		let negbit = (comp >> 9) & 0x1;

		comp >>= 10;

		let mut val = std::f32::consts::FRAC_1_SQRT_2 * (mag as f32) / (C_MASK as f32);

		val = if negbit == 1 { -val } else { val };

		rotation[i] = val;
		sum_squares += val * val;
	}
	rotation[i_largest] = (1.0_f32 - sum_squares).max(0.0_f32).sqrt();

	for i in 0..3 {
		rotation[i] *= flip_q[i];
	}
}

pub fn pack_quaternion_smallest_three(rotation: &[f32; 4], flip_q: [f32; 3]) -> [u8; 4] {
	let mut rot_normed = normalize_quaternion(rotation);

	rot_normed[0] *= flip_q[0];
	rot_normed[1] *= flip_q[1];
	rot_normed[2] *= flip_q[2];

	let mut i_largest = 0_usize;

	for i in 1..4 {
		if rot_normed[i].abs() > rot_normed[i_largest].abs() {
			i_largest = i;
		}
	}
	let negate = rot_normed[i_largest] < 0.0;

	let c_mask = (1_u32 << 9) - 1;
	let mut comp: u32 = i_largest as u32;

	for i in 0..4 {
		if i == i_largest {
			continue;
		}
		let negbit = if (rot_normed[i] < 0.0) ^ negate {
			1_u32
		} else {
			0_u32
		};
		let mag = ((c_mask as f32) * (rot_normed[i].abs() / FRAC_1_SQRT_2) + 0.5).floor()
			as u32;
		let mag = mag.min(c_mask);

		comp = (comp << 10) | (negbit << 9) | mag;
	}
	let r = {
		let mut r = [0_u8; 4];

		r[0] = (comp & 0xff) as u8;
		r[1] = ((comp >> 8) & 0xff) as u8;
		r[2] = ((comp >> 16) & 0xff) as u8;
		r[3] = ((comp >> 24) & 0xff) as u8;
		r
	};
	r
}

#[inline]
pub fn sigmoid(x: f32) -> f32 {
	1.0 / (1.0 + (-x).exp())
}

#[inline]
pub fn inv_sigmoid(mut x: f32) -> f32 {
	// clamp to avoid division by zero at x == 1
	x = x.clamp(1e-6, 1.0 - 1e-6);

	(x / (1.0_f32 - x)).ln()
}

#[inline]
pub fn unquantize_sh(sh: u8) -> f32 {
	(sh as f32 - 128.0_f32) / 128.0_f32
}

#[inline]
pub fn quantize_sh(mut sh: f32, step: i32) -> u8 {
	sh = (sh * 128.0 + 128.0).round();
	let quantized = ((sh as i32 / step) * step).clamp(0, 255);

	quantized as u8
}

pub fn normalize_quaternion(q: &[f32; 4]) -> [f32; 4] {
	let norm_sq = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];

	if norm_sq < f32::EPSILON {
		return [0.0, 0.0, 0.0, 1.0];
	}
	let inv_norm = 1.0 / norm_sq.sqrt();
	[
		q[0] * inv_norm,
		q[1] * inv_norm,
		q[2] * inv_norm,
		q[3] * inv_norm,
	]
}

#[inline]
pub fn to_u8(x: f32) -> u8 {
	x.clamp(0.0, 255.0).round() as u8
}

#[cfg(test)]
mod tests {
	use super::*;
	use approx::assert_relative_eq;
	use rstest::rstest;

	#[rstest]
	#[case(0, 0)]
	#[case(1, 0)]
	#[case(2, 0)]
	#[case(3, 1)]
	#[case(7, 1)]
	#[case(8, 2)]
	#[case(14, 2)]
	#[case(15, 3)]
	#[case(255, 3)]
	fn test_degree_for_dim(#[case] dim: u8, #[case] expected: u8) {
		assert_eq!(degree_for_dim(dim), expected);
	}

	#[rstest]
	#[case(0, 0)]
	#[case(1, 3)]
	#[case(2, 8)]
	#[case(3, 15)]
	#[case(4, 0)] // out-of-range degree returns 0
	#[case(255, 0)]
	fn test_dim_for_degree(#[case] degree: u8, #[case] expected: u8) {
		assert_eq!(dim_for_degree(degree), expected);
	}

	/// Verify that degree_for_dim(dim_for_degree(d)) == d for valid degrees.
	#[rstest]
	#[case(0)]
	#[case(1)]
	#[case(2)]
	#[case(3)]
	fn test_degree_dim_roundtrip(#[case] degree: u8) {
		let dim = dim_for_degree(degree);

		if dim > 0 {
			assert_eq!(degree_for_dim(dim), degree);
		}
	}

	#[rstest]
	#[case(0.0, 0.5)]
	#[case(100.0, 1.0)]
	#[case(-100.0, 0.0)]
	fn test_sigmoid_known_values(#[case] x: f32, #[case] expected: f32) {
		assert_relative_eq!(sigmoid(x), expected, epsilon = 1e-5);
	}

	#[test]
	fn test_sigmoid_monotonic() {
		let values: Vec<f32> = (-50..=50).map(|x| sigmoid(x as f32 * 0.1)).collect();

		for w in values.windows(2) {
			assert!(w[1] >= w[0], "sigmoid must be monotonically increasing");
		}
	}

	#[rstest]
	#[case(0.01)]
	#[case(0.1)]
	#[case(0.25)]
	#[case(0.5)]
	#[case(0.75)]
	#[case(0.9)]
	#[case(0.99)]
	fn test_sigmoid_inv_sigmoid_roundtrip(#[case] x: f32) {
		let recovered = sigmoid(inv_sigmoid(x));

		assert_relative_eq!(recovered, x, epsilon = 1e-5);
	}

	#[test]
	fn test_inv_sigmoid_clamps_extremes() {
		let v0 = inv_sigmoid(0.0);
		let v1 = inv_sigmoid(1.0);

		assert!(v0.is_finite());
		assert!(v1.is_finite());
		assert!(v0 < 0.0);
		assert!(v1 > 0.0);
	}

	#[rstest]
	#[case(128, 0.0)]
	#[case(0, -1.0)]
	#[case(255, 127.0 / 128.0)]
	fn test_unquantize_sh(#[case] input: u8, #[case] expected: f32) {
		assert_relative_eq!(unquantize_sh(input), expected, epsilon = 1e-6);
	}

	#[rstest]
	#[case(0.0, 8, 128)]
	#[case(100.0, 1, 255)]
	#[case(-100.0, 1, 0)]
	fn test_quantize_sh(#[case] input: f32, #[case] step: i32, #[case] expected: u8) {
		assert_eq!(quantize_sh(input, step), expected);
	}

	#[test]
	fn test_quantize_unquantize_sh_roundtrip_step1() {
		for i in 0..=255_u8 {
			let f = unquantize_sh(i);
			let back = quantize_sh(f, 1);

			assert_eq!(back, i, "roundtrip failed for {}", i);
		}
	}

	#[rstest]
	#[case([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0])]
	#[case([2.0, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0])]
	#[case([0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0])]
	fn test_normalize_quaternion_expected(#[case] input: [f32; 4], #[case] expected: [f32; 4]) {
		let n = normalize_quaternion(&input);

		for i in 0..4 {
			assert_relative_eq!(n[i], expected[i], epsilon = 1e-6);
		}
	}

	#[rstest]
	#[case([0.0, 0.0, 0.0, 1.0])]
	#[case([2.0, 0.0, 0.0, 0.0])]
	#[case([1.0, 2.0, 3.0, 4.0])]
	#[case([0.5, 0.5, 0.5, 0.5])]
	fn test_normalize_quaternion_is_unit(#[case] input: [f32; 4]) {
		let n = normalize_quaternion(&input);
		let norm_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2] + n[3] * n[3];

		assert_relative_eq!(norm_sq, 1.0, epsilon = 1e-6);
	}

	#[rstest]
	#[case(0.0, 0)]
	#[case(127.5, 128)]
	#[case(255.0, 255)]
	#[case(-10.0, 0)]
	#[case(300.0, 255)]
	#[case(0.4, 0)]
	#[case(0.6, 1)]
	fn test_to_u8(#[case] input: f32, #[case] expected: u8) {
		assert_eq!(to_u8(input), expected);
	}

	#[rstest]
	#[case([0.0, 0.0, 0.0, 1.0])]
	#[case([1.0, 0.0, 0.0, 0.0])]
	#[case([0.0, 1.0, 0.0, 0.0])]
	#[case([0.0, 0.0, 1.0, 0.0])]
	#[case([0.5, 0.5, 0.5, 0.5])]
	fn test_pack_unpack_quaternion_smallest_three_roundtrip(#[case] input: [f32; 4]) {
		let no_flip = [1.0_f32, 1.0, 1.0];
		let normed = normalize_quaternion(&input);
		let packed = pack_quaternion_smallest_three(&normed, no_flip);
		let mut unpacked = [0.0_f32; 4];

		unpack_quaternion_smallest_three(&mut unpacked, &packed);

		let dot: f32 = normed.iter().zip(unpacked.iter()).map(|(a, b)| a * b).sum();

		assert_relative_eq!(dot.abs(), 1.0, epsilon = 0.01);
	}

	#[test]
	fn test_pack_unpack_quaternion_with_flip() {
		let q = [0.1_f32, 0.2, 0.3, 0.9];
		let flip = [-1.0_f32, 1.0, -1.0];
		let packed = pack_quaternion_smallest_three(&q, flip);
		let mut unpacked = [0.0_f32; 4];

		unpack_quaternion_smallest_three_with_flip(&mut unpacked, &packed, flip);

		let norm = normalize_quaternion(&q);
		let dot: f32 = norm.iter().zip(unpacked.iter()).map(|(a, b)| a * b).sum();

		assert_relative_eq!(dot.abs(), 1.0, epsilon = 0.02);
	}

	#[rstest]
	#[case([128_u8, 128, 128], [0.0, 0.0, 0.0, 1.0])]
	fn test_unpack_quaternion_first_three_identity(
		#[case] r: [u8; 3],
		#[case] mut expected: [f32; 4],
	) {
		unpack_quaternion_first_three(&mut expected, &r);

		assert_relative_eq!(expected[0], 0.0, epsilon = 0.01);
		assert_relative_eq!(expected[1], 0.0, epsilon = 0.01);
		assert_relative_eq!(expected[2], 0.0, epsilon = 0.01);
		assert_relative_eq!(expected[3], 1.0, epsilon = 0.01);
	}

	#[test]
	fn test_unpack_quaternion_first_three_w_derived() {
		let r = [200_u8, 150, 140];
		let mut rot = [0.0_f32; 4];

		unpack_quaternion_first_three(&mut rot, &r);

		let norm = (rot[0] * rot[0] + rot[1] * rot[1] + rot[2] * rot[2] + rot[3] * rot[3])
			.sqrt();

		assert_relative_eq!(norm, 1.0, epsilon = 0.01);
		assert!(rot[3] >= 0.0, "w component must be non-negative");
	}
}
