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
	pub fn decompress_end<C, D>(compressed: C, mut decompressed: D) -> Result<()>
	where
		C: AsRef<[u8]>,
		D: AsMut<Vec<u8>>,
	{
		let mut gz_decoder = GzDecoder::new(compressed.as_ref());

		gz_decoder
			.read_to_end(decompressed.as_mut())
			.with_context(|| "unable to decompress to end")?;

		Ok(())
	}

	/// Decompress gzip-compressed data into the given buffer.
	#[inline]
	pub fn decompress<C, D>(compressed: C, mut decompressed: D) -> Result<()>
	where
		C: AsRef<[u8]>,
		D: AsMut<[u8]>,
	{
		let mut gz_decoder = GzDecoder::new(compressed.as_ref());

		gz_decoder
			.read(decompressed.as_mut())
			.with_context(|| "unable to decompress into the given buffer")?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::gzip;

	#[test]
	fn test_compress_decompress_roundtrip() {
		let original = b"Hello, Gaussian Splatting! This is a test of gzip roundtrip.";
		let mut compressed = Vec::new();

		gzip::compress_bytes(original.as_slice(), &mut compressed)
			.expect("compression failed");

		// Compressed data should differ from original
		assert_ne!(compressed.as_slice(), original.as_slice());
		assert!(!compressed.is_empty());

		let mut decompressed = Vec::new();

		gzip::decompress_end(&compressed, &mut decompressed).expect("decompression failed");

		assert_eq!(decompressed.as_slice(), original.as_slice());
	}

	#[test]
	fn test_compress_decompress_large_data() {
		let original: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
		let mut compressed = Vec::new();

		gzip::compress_bytes(&original, &mut compressed).expect("compression failed");

		assert!(compressed.len() < original.len());

		let mut decompressed = Vec::new();

		gzip::decompress_end(&compressed, &mut decompressed).expect("decompression failed");

		assert_eq!(decompressed, original);
	}

	#[test]
	fn test_compress_empty_data() {
		let original: &[u8] = &[];
		let mut compressed = Vec::new();

		gzip::compress_bytes(original, &mut compressed)
			.expect("compression of empty data failed");

		// Even empty data produces gzip header/footer
		assert!(!compressed.is_empty());

		let mut decompressed = Vec::new();

		gzip::decompress_end(&compressed, &mut decompressed)
			.expect("decompression of empty data failed");

		assert!(decompressed.is_empty());
	}

	#[test]
	fn test_decompress_invalid_data() {
		let bad_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
		let mut decompressed = Vec::new();

		let result = gzip::decompress_end(&bad_data, &mut decompressed);

		assert!(result.is_err());
	}

	#[test]
	fn test_decompress_into_fixed_buffer() {
		let original = b"Short text for fixed buffer decompress.";
		let mut compressed = Vec::new();

		gzip::compress_bytes(original.as_slice(), &mut compressed)
			.expect("compression failed");

		let mut buf = vec![0_u8; original.len() + 64];

		gzip::decompress(&compressed, buf.as_mut_slice())
			.expect("decompression into fixed buffer failed");

		assert_eq!(&buf[..original.len()], original.as_slice());
	}

	#[test]
	fn test_compress_bytes_clears_output() {
		let mut compressed = vec![0xFF; 100];

		gzip::compress_bytes(b"data", &mut compressed).expect("compression failed");

		// Output should not start with the pre-filled 0xFF bytes
		// (gzip magic is 0x1F 0x8B)
		assert_eq!(compressed[0], 0x1F);
		assert_eq!(compressed[1], 0x8B);
	}
}
