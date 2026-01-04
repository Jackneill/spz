// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::io::{BufRead, BufReader, Read};

use flate2::{
	Compression,
	bufread::{GzDecoder, GzEncoder},
};

use crate::errors::SpzError;

/// Compress data using gzip compression.
#[inline]
pub fn to_gzip_bytes(decompressed: &[u8], compressed: &mut Vec<u8>) -> Result<(), SpzError> {
	compressed.clear();
	compressed.reserve(decompressed.len() / 4);

	let reader = BufReader::new(decompressed);

	to_gzip(reader, compressed)
}

/// Compress data using gzip compression.
#[inline]
pub fn to_gzip<B>(decompressed: B, compressed: &mut Vec<u8>) -> Result<(), SpzError>
where
	B: BufRead,
{
	let mut encoder = GzEncoder::new(decompressed, Compression::default());

	encoder.read_to_end(compressed)?;

	Ok(())
}

/// Decompress gzip-compressed data into the given buffer.
#[inline]
pub fn gzip_to_bytes<C, D>(compressed: C, mut decompressed: D) -> Result<(), SpzError>
where
	C: AsRef<[u8]>,
	D: AsMut<Vec<u8>>,
{
	let mut gz_decoder = GzDecoder::new(compressed.as_ref());

	gz_decoder.read_to_end(decompressed.as_mut())?;

	Ok(())
}
