# SPZ Cap'n Proto

<div align="center">
	<b>Rust Cap'n Proto</b> integration for the <b>.SPZ</b> file format.
</div>
<br>

* For more information about the SPZ file format, see [https://github.com/Jackneill/spz](https://github.com/Jackneill/spz)
* This crate's repo is located at [https://github.com/Jackneill/spz/tree/main/crates/spz-capnproto-rust](https://github.com/Jackneill/spz/tree/main/crates/spz-capnproto-rust)

## Overview

This crate generates Rust code from the [spz.capnp](https://github.com/Jackneill/spz/blob/main/capnproto/schemas/spz.capnp) schema, providing types and methods for interacting with SPZ data structures via Cap'n Proto.

## What is SPZ?

SPZ is a compressed file format for 3D Gaussian Splats, designed by Niantic.
It provides efficient storage of Gaussian Splat data with configurable
spherical harmonics degrees and coordinate system support.

About 10x smaller than the PLY equivalent with virtually no perceptible loss in
visual quality.

See [docs/SPZ_SPEC_v3.md](docs/SPZ_SPEC_v3.md) for more information.

## Usage

```toml
[dependencies]
spz-capnproto = { version = "*", default-features = false, features = [
	# Convenience fns and trait impls for converting between spz and capnp types.
	# Also re-exports the used `spz` crate.
	"spz",
] }
```

## License

<!-- REUSE-IgnoreStart -->

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 	* `SPDX-License-Identifier: Apache-2.0`
* MIT license ([LICENSE-MIT](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
at your option.
 	* `SPDX-License-Identifier: MIT`

<!-- REUSE-IgnoreEnd -->

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the _Apache-2.0_ license, shall
be dual licensed as above, without any additional terms or conditions.

<a href="https://app.fossa.com/projects/git%2Bgithub.com%2FJackneill%2Fspz?ref=badge_large&issueType=license" alt="FOSSA Status">
	<img alt="FOSSA Scan" src="https://app.fossa.com/api/projects/git%2Bgithub.com%2FJackneill%2Fspz.svg?type=large&issueType=license"/>
</a>
