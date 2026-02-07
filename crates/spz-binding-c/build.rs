// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::PathBuf;
use std::process::{Command, ExitCode};

use anyhow::{Context, Result};
use cbindgen::Config;

fn main() -> Result<ExitCode> {
	// non-existent file to always trigger the build script
	//println!("cargo::rerun-if-changed=NULL");
	println!("cargo::rerun-if-changed=src/lib.rs");
	println!("cargo::rerun-if-changed=cbindgen.toml");

	let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
	let header_path = crate_dir.join("include/spz.h");

	cbindgen::Builder::new()
		.with_crate(&crate_dir)
		.with_config(Config::from_file("cbindgen.toml").unwrap())
		.generate()?
		.write_to_file(&header_path);

	Command::new("clang-format")
		.args(["-i", "--style=file:../../.clang-format"])
		.arg(&header_path)
		.status()
		.map(|ret| {
			if ret.success() {
				ExitCode::SUCCESS
			} else {
				ExitCode::from(1)
			}
		})
		.with_context(|| "clang-format returned error")
}
