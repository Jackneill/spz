// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{fs::File, path::Path};

use anyhow::Context;
use anyhow::Result;
use memmap2::Mmap;

/// Memory-maps a file for efficient read-only access.
#[inline]
pub fn mmap<F>(filepath: F) -> Result<Mmap>
where
	F: AsRef<Path>,
{
	let infile = File::open(&filepath)?;

	unsafe { Mmap::map(&infile).with_context(|| "unable to open file with mmap()") }
}

/// Memory-maps a file for efficient read-only access with a specified range.
#[inline]
pub fn mmap_range<F>(filepath: F, offset: usize, len: usize) -> Result<Mmap>
where
	F: AsRef<Path>,
{
	let infile = File::open(&filepath)?;

	unsafe {
		memmap2::MmapOptions::new()
			.offset(offset as u64)
			.len(len)
			.map(&infile)
			.with_context(|| {
				format!(
					"unable to open file with mmap() (range {}..{}, len {})",
					offset,
					offset + len,
					len
				)
			})
	}
}
