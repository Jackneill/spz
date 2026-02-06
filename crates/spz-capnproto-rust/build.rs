use std::process::Child;

use anyhow::{Context, Result, bail};

/// A helper macro to give `cargofmt_generated_impl` variadic arguments.
macro_rules! cargofmt_generated {
	($($e:expr),+ $(,)?) => {
		cargofmt_generated_impl(vec![$($e),+])
	};
}

fn main() -> Result<()> {
	let out_dir = std::env::var("OUT_DIR")?;
	//let _cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
	let schemas_dir = "../../capnproto/schemas";
	let spz_schema_path = format!("{}/spz.capnp", schemas_dir);

	//println!("cargo::rerun-if-changed={}", spz_schema_path);
	// non-existent file to always trigger the build script
	println!("cargo::rerun-if-changed=NULL");

	is_capnpc_bin_exists()?;

	::capnpc::CompilerCommand::new()
		.src_prefix(&schemas_dir)
		.file(&spz_schema_path)
		.run()
		.with_context(|| {
			format!("error compiling capnproto schemas ({})", spz_schema_path)
		})?;

	cargofmt_generated! {
		format!("{}/spz_capnp.rs", out_dir),
	}
	.with_context(|| "error formatting generated files with cargo fmt")?;

	Ok(())
}

/// Checks whether the `capnpc` binary is found in `$PATH`.
#[inline]
fn is_capnpc_bin_exists() -> Result<()> {
	let status = std::process::Command::new("capnpc")
		.arg("--version")
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.status();

	match status {
		Ok(s) if s.success() => Ok(()),
		_ => bail!("the `capnpc` binary was not found in $PATH; \
			 install the Cap'n Proto compiler (https://capnproto.org/install.html)"),
	}
}

/// Formats the given files with `cargo fmt`.
#[inline]
fn cargofmt_generated_impl(paths: Vec<String>) -> Result<Child> {
	std::process::Command::new("cargo")
		.arg("fmt")
		.arg("--")
		.arg(paths.join(" ").as_str())
		.spawn()
		.with_context(|| "error spawning cargo fmt")
}
