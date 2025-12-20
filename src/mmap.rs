// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::{
	fs::{File, OpenOptions},
	path::Path,
};

use anyhow::Context;
use anyhow::Result;
use memmap2::{Mmap, MmapMut};

pub fn open<F>(file: F) -> Result<Mmap>
where
	F: AsRef<Path>,
{
	let infile = File::open(&file)?;

	unsafe { Mmap::map(&infile).with_context(|| "unable to open file with mmap()") }
}

pub fn write<P, D>(path: P, data: D) -> Result<()>
where
	P: AsRef<Path>,
	D: AsRef<[u8]>,
{
	let file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(&path)?;

	unsafe {
		MmapMut::map_mut(&file)
			.with_context(|| "unable to mmap file for writing")?
			.copy_from_slice(data.as_ref());
	}
	Ok(())
}
