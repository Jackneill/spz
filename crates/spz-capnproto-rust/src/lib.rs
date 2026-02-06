#[cfg(feature = "spz")]
pub use spz;

pub mod generated {
	pub use super::spz_capnp::*;
}

::capnp::generated_code!(pub mod spz_capnp);
