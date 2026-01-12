use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpzError {
	#[error("data is empty")]
	DataIsEmpty,
	#[error("invalid fractional bits (= {0}): {1}")]
	InvalidFractionalBits(i32, i32),
	#[error("index out of bounds: {0}")]
	IndexOutOfBounds(usize),
	#[error(transparent)]
	IoError(#[from] std::io::Error),
	#[error("invalid magic number in packed gaussians header")]
	InvalidMagicNumber,
	#[error("inconsistent sizes")]
	InconsistentSizes,
	#[error("unsupported version: {0}")]
	UnsupportedVersion(i32),
	#[error("unsupported spherical harmonics degree: {0}")]
	UnsupportedSphericalHarmonicsDegree(u8),
	#[error("{0}")]
	LoadPackedError(String),
	#[error("unsupported: {0}")]
	UnsupportedFormat(String),
}
