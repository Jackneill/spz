<h1 align="center">SPZ<span></span></h1>

<div align="center">Rust implementation of the .SPZ file format and CLI tools.</div>
&nbsp;
<div align="center"><b>WIP</b></div>
&nbsp;
<p align="center">
	<a href="https://crates.io/crates/spz">
		<img alt="Crates.io Version" src="https://img.shields.io/crates/v/spz?style=for-the-badge&link=https%3A%2F%2Fcrates.io%2Fcrates%2Fspz">
	</a>
	<a href="https://docs.rs/spz">
		<img alt="docs.rs" src="https://img.shields.io/docsrs/spz?style=for-the-badge&label=docs.rs&link=docs.rs%2Fspz">
	</a>
	<a href="https://lib.rs/crates/spz">
		<img alt="lib.rs" src="https://img.shields.io/badge/spz-librs?style=for-the-badge&label=Lib.rs&link=https%3A%2F%2Flib.rs%2Fcrates%2Fspz">
	</a>
	<img alt="GitHub Tag" src="https://img.shields.io/github/v/tag/Jackneill/spz?style=for-the-badge">
	<br>
	<img alt="GitHub CI" src="https://img.shields.io/github/check-runs/Jackneill/spz/main?style=for-the-badge&label=CI%3Amain">
	<img alt="Deps" src="https://img.shields.io/deps-rs/repo/github/Jackneill/spz?style=for-the-badge">
	<img alt="GitHub Last Commit" src="https://img.shields.io/github/last-commit/Jackneill/spz/main?style=for-the-badge">
	<br>
	<a href="https://codspeed.io/Jackneill/spz">
		<img alt="CodSpeed" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fcodspeed.io%2Fbadge.json&style=for-the-badge" />
	</a>
	<a href="https://codecov.io/github/Jackneill/spz">
		<img alt="CodeCov" src="https://codecov.io/github/Jackneill/spz/graph/badge.svg?style=for-the-badge&token=10QLWY4MWG"/>
	</a>
	<br>
	<a href="./LICENSE-APACHE">
		<img alt="GitHub License" src="https://img.shields.io/github/license/Jackneill/spz?style=for-the-badge&label=LICENSE">
	</a>
	<a href="./LICENSE-MIT">
		<img alt="GitHub License MIT" src="https://img.shields.io/badge/MIT-LICENSE?style=for-the-badge&label=LICENSE">
	</a>
	<br>
	<a href="https://app.fossa.com/projects/git%2Bgithub.com%2FJackneill%2Fspz?ref=badge_shield&issueType=license" alt="FOSSA Status">
		<img alt="FOSSA Status" src="https://app.fossa.com/api/projects/git%2Bgithub.com%2FJackneill%2Fspz.svg?type=shield&issueType=license"/>
	</a>
	<a href="https://app.fossa.com/projects/git%2Bgithub.com%2FJackneill%2Fspz?ref=badge_shield&issueType=security" alt="FOSSA Status">
		<img alt="FOSSA Security" src="https://app.fossa.com/api/projects/git%2Bgithub.com%2FJackneill%2Fspz.svg?type=shield&issueType=security"/>
	</a>
</p>

## What is SPZ?

SPZ is a compressed file format for 3D Gaussian Splats, designed by Niantic.
It provides efficient storage of Gaussian Splat data with configurable
spherical harmonics degrees and coordinate system support.

See [docs/SPZ.md](docs/SPZ.md) for more information.

## Usage

```toml
spz = { version = "0.0.3", default-features = false, features = [] }
```

```rust
use spz::prelude::*;
```

## Examples

```sh
cargo run --example load_spz
```

## Quick Start

```rust
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::path::{Path, PathBuf};

use anyhow::Result;
use spz::{GaussianSplat, UnpackOptions};

fn main() -> Result<()> {
	let mut sample_spz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	sample_spz.push("assets/racoonfamily.spz");

	let _gs = spz::GaussianSplat::builder()
		.filepath(sample_spz)
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(spz::CoordinateSystem::default())
				.build(),
		)
		.load()?;

	Ok(())
}

#[allow(unused)]
async fn load_spz_async<P>(spz_file: P) -> Result<GaussianSplat>
where
	P: AsRef<Path>,
{
	let gs = spz::GaussianSplat::builder()
		.filepath(spz_file)
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(spz::CoordinateSystem::default())
				.build(),
		)
		.load_async()
		.await?;

	Ok(gs)
}
```

## API

### Overview

```rust
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GaussianSplat {
	pub num_points: i32,
	pub spherical_harmonics_degree: i32,
	pub antialiased: bool,
	pub positions: Vec<f32>,
	pub scales: Vec<f32>,
	pub rotations: Vec<f32>,
	pub alphas: Vec<f32>,
	pub colors: Vec<f32>,
	pub spherical_harmonics: Vec<f32>,
}
```

## Benches

### Pre-Requisites

* Install `gnuplot` for html reports.
* [Install `nextest` runner](https://nexte.st/docs/installation/pre-built-binaries/).

### Run

```sh
just bench
```

* The html report of the benchmark can be found under `target/criterion/report/index.html`.
* View Benchmark and Profiling data on [CodSpeed](https://codspeed.io/Jackneill/spz), (CI runs).

## Code Coverage

<a href="https://codecov.io/github/Jackneill/spz">
	<img alt="CodeCov Grid" src="https://codecov.io/github/Jackneill/spz/graphs/tree.svg?token=10QLWY4MWG"/>
</a>

## Development

### Pre-Requisites

* Install the `mold` linker: <https://github.com/rui314/mold>
* [Install `nextest`](https://nexte.st/docs/installation/pre-built-binaries/)

## Documentation

Further documentation is available under `./docs`.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 	* `SPDX-License-Identifier: Apache-2.0`
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
at your option.
 	* `SPDX-License-Identifier: MIT`

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the _Apache-2.0_ license, shall
be dual licensed as above, without any additional terms or conditions.

<a href="https://app.fossa.com/projects/git%2Bgithub.com%2FJackneill%2Fspz?ref=badge_large&issueType=license" alt="FOSSA Status">
	<img alt="FOSSA Scan" src="https://app.fossa.com/api/projects/git%2Bgithub.com%2FJackneill%2Fspz.svg?type=large&issueType=license"/>
</a>
