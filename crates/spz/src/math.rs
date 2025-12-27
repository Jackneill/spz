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

pub fn half_to_float(h: u16) -> f32 {
	let sgn = ((h >> 15) & 0x1) as u32;
	let exponent = ((h >> 10) & 0x1f) as i32;
	let mantissa = (h & 0x3ff) as u32;
	let sign_mul = if sgn == 1 { -1.0_f32 } else { 1.0_f32 };

	if exponent == 0 {
		return sign_mul * 2_f32.powf(-14.0) * (mantissa as f32) / 1024.0;
	}
	if exponent == 31 {
		return if mantissa == 0 {
			sign_mul * f32::INFINITY
		} else {
			f32::NAN
		};
	}
	let exp = exponent as f32 - 15.0;

	sign_mul * 2_f32.powf(exp) * (1.0 + (mantissa as f32) / 1024.0)
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
		if i != i_largest {
			let mag = comp & C_MASK;
			let negbit = (comp >> 9) & 0x1;

			comp >>= 10;

			let mut val =
				std::f32::consts::FRAC_1_SQRT_2 * (mag as f32) / (C_MASK as f32);

			if negbit == 1 {
				val = -val;
			}
			rotation[i] = val;
			sum_squares += val * val;
		}
	}
	rotation[i_largest] = (1.0_f32 - sum_squares).max(0.0_f32).sqrt();

	for i in 0..3 {
		rotation[i] *= flip_q[i];
	}
}

pub fn pack_quaternion_smallest_three(rotation: &[f32; 4], flip_q: [f32; 3]) -> [u8; 4] {
	let mut q = normalize_quaternion(rotation);

	q[0] *= flip_q[0];
	q[1] *= flip_q[1];
	q[2] *= flip_q[2];

	let mut i_largest = 0_usize;

	for i in 1..4 {
		if q[i].abs() > q[i_largest].abs() {
			i_largest = i;
		}
	}
	let negate = q[i_largest] < 0.0;

	let c_mask = (1_u32 << 9) - 1;
	let mut comp: u32 = i_largest as u32;

	for i in 0..4 {
		if i == i_largest {
			continue;
		}
		let negbit = if (q[i] < 0.0) ^ negate { 1_u32 } else { 0_u32 };

		let mag = ((c_mask as f32) * (q[i].abs() / FRAC_1_SQRT_2) + 0.5).floor() as u32;
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
