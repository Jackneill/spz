use anyhow::{Context, Result};

fn main() -> Result<()> {
	println!("cargo::rerun-if-changed=../../capnproto/schemas/spz.capnp");

	::capnpc::CompilerCommand::new()
		.file("../../capnproto/schemas/spz.capnp")
		.run()
		.with_context(|| "error compiling schema")
}
