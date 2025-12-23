// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{fs::File, path::Path};

use anyhow::Context;
use anyhow::Result;
use memmap2::Mmap;

pub fn open<F>(file: F) -> Result<Mmap>
where
	F: AsRef<Path>,
{
	let infile = File::open(&file)?;

	unsafe { Mmap::map(&infile).with_context(|| "unable to open file with mmap()") }
}
