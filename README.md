<h1 align="center">SPZ<span></span></h1>

<div align="center"><b>Rust</b> and <b>Python</b> implementation of the <b>.SPZ</b> file format (v3) and <b>CLI</b> tools.</div>
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
	<hr>
	<br>
	<a href='https://flathub.org/apps/org.jackneill.spz'>
		<img width='160' alt='Get it on Flathub' src='https://flathub.org/api/badge?locale=en'/>
	</a>
</p>

## What is SPZ?

SPZ is a compressed file format for 3D Gaussian Splats, designed by Niantic.
It provides efficient storage of Gaussian Splat data with configurable
spherical harmonics degrees and coordinate system support.

See [docs/SPZ.md](docs/SPZ.md) for more information.

## CLI

```sh
$ path/to/spz info assets/racoonfamily.spz

GaussianSplat={num_points=932560, sh_degree=3, antialiased=true, median_ellipsoid_volume=0.0000000046213082, bbox=[x=-281.779541 to 258.382568, y=-240.000000 to 240.000000, z=-240.000000 to 240.000000]}
```

## Rust

## Usage

```toml
spz = { version = "0.0.6", default-features = false, features = [] }
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
use spz::{coord::CoordinateSystem, prelude::GaussianSplat, unpacked::UnpackOptions};

fn main() -> Result<()> {
	let mut sample_spz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	sample_spz.push("assets/racoonfamily.spz");

	let _gs = GaussianSplat::builder()
		.filepath(sample_spz)
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(CoordinateSystem::default())
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
	GaussianSplat::builder()
		.filepath(spz_file)
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(CoordinateSystem::default())
				.build(),
		)
		.load_async()
		.await
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

## Tests

### Pre-Requisites

* [Install `nextest` runner](https://nexte.st/docs/installation/pre-built-binaries/).

## Benches

### Pre-Requisites

* `cargo install cargo-criterion`.
* Install `gnuplot` for html reports.

### Run

```sh
just bench
```

* The html report of the benchmark can be found under `./target/criterion/report/index.html`.
* View Benchmark and Profiling data on [CodSpeed](https://codspeed.io/Jackneill/spz), (from CI runs).

## Test Code Coverage

<a href="https://codecov.io/github/Jackneill/spz">
	<img alt="CodeCov Grid" src="https://codecov.io/github/Jackneill/spz/graphs/tree.svg?token=10QLWY4MWG" width="300"/>
</a>

## Build

### Pre-Requisites

* Install the `mold` linker: <https://github.com/rui314/mold>

## Python

## Usage

```sh
uvx pip install spz
```

```toml
# pyproject.toml

[project]
dependencies = [
    "spz",
]
```

## Examples

```py
import spz

# Load from file
splat = spz.load("scene.spz")
# or
splat = spz.GaussianSplat.load("scene.spz", coordinate_system=spz.CoordinateSystem.RUB)

# Access properties
print(f"{splat.num_points:,} points")
print(f"center: {splat.bbox.center}")
print(f"size: {splat.bbox.size}")

# Access data (flat arrays)
positions = splat.positions  # [x1, y1, z1, x2, y2, z2, ...]
scales = splat.scales
rotations = splat.rotations
alphas = splat.alphas
colors = splat.colors
sh = splat.spherical_harmonics

data = splat.to_bytes()
splat2 = spz.GaussianSplat.from_bytes(data)  # Serialize

splat.save("output.spz")  # Save to file

new_splat = spz.GaussianSplat(
    positions=[0.0, 0.0, 0.0, 1.0, 2.0, 3.0],  # flat array
    scales=[-5.0] * 6,
    rotations=[1.0, 0.0, 0.0, 0.0] * 2,
    alphas=[0.5, 0.8],
    colors=[255.0, 0.0, 0.0, 0.0, 255.0, 0.0],
)

with spz.modified_splat("scene.spz", "scene_rotated.spz") as splat:
    splat.rotate_180_deg_about_x()
```

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
