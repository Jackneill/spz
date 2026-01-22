// SPDX-License-Identifier: Apache-2.0 OR MIT

pub mod gzip {
	use std::io::{BufRead, BufReader, Read};

	use anyhow::Context;
	use anyhow::Result;
	use flate2::{
		Compression,
		bufread::{GzDecoder, GzEncoder},
	};

	/// Compress data using gzip compression.
	#[inline]
	pub fn compress_bytes(decompressed: &[u8], compressed: &mut Vec<u8>) -> Result<()> {
		compressed.clear();
		compressed.reserve(decompressed.len() / 4);

		let reader = BufReader::new(decompressed);

		compress(reader, compressed)
	}

	/// Compress data using gzip compression.
	#[inline]
	pub fn compress<B>(decompressed: B, compressed: &mut Vec<u8>) -> Result<()>
	where
		B: BufRead,
	{
		let mut encoder = GzEncoder::new(decompressed, Compression::default());

		encoder.read_to_end(compressed)
			.with_context(|| "unable to decompress")?;

		Ok(())
	}

	/// Decompress gzip-compressed data into the given buffer.
	#[inline]
	pub fn decompress_bytes<C, D>(compressed: C, mut decompressed: D) -> Result<()>
	where
		C: AsRef<[u8]>,
		D: AsMut<Vec<u8>>,
	{
		let mut gz_decoder = GzDecoder::new(compressed.as_ref());

		gz_decoder
			.read_to_end(decompressed.as_mut())
			.with_context(|| "unable to decompress")?;

		Ok(())
	}
}
