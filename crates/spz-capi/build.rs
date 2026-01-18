// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;

use anyhow::Result;
use cbindgen::Config;

fn main() -> Result<()> {
	println!("cargo:rerun-if-changed=src/lib.rs");
	println!("cargo:rerun-if-changed=cbindgen.toml");

	let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

	cbindgen::Builder::new()
		.with_crate(crate_dir)
		.with_config(Config::from_file("cbindgen.toml").unwrap())
		.generate()?
		.write_to_file("include/spz.h");

	Ok(())
}
