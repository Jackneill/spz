// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{fs::File, path::Path};

use crate::errors::SpzError;
use memmap2::Mmap;

pub fn open<F>(file: F) -> Result<Mmap, SpzError>
where
	F: AsRef<Path>,
{
	let infile = File::open(&file)?;

	unsafe {
		Mmap::map(&infile).map_err(|_| {
			SpzError::LoadPackedError("unable to open file with mmap()".to_string())
		})
	}
}
