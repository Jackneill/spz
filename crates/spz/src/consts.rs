// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::str::FromStr;

use anyhow::{Error, anyhow};
use strum::EnumIter;

/// Scale factor for DC color components.
///
/// To convert to RGB, we should multiply by `0.282`, but it can
/// be useful to represent base colors that are out of range if the higher
/// spherical harmonics bands bring them back into range so we multiply by a
/// smaller value.
pub const COLOR_SCALE: f32 = 0.15;

/// Standard file extensions for SPZ files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Extensions {
	SPZ,
}

impl FromStr for Extensions {
	type Err = Error;

	#[inline]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			"spz" => Ok(Extensions::SPZ),
			_ => Err(anyhow!("invalid extension: {}", s)),
		}
	}
}

impl Extensions {
	#[inline]
	pub const fn as_str(&self) -> &'static str {
		match self {
			Extensions::SPZ => "spz",
		}
	}
}

impl From<&str> for Extensions {
	#[inline]
	fn from(s: &str) -> Self {
		Self::from_str(s).expect("invalid extension")
	}
}

impl std::fmt::Display for Extensions {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Extensions::SPZ => "spz",
		};
		write!(f, "{}", s)
	}
}
