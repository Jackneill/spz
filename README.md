<h1 align="center">SPZ<span></span></h1>

<div align="center"><b>Rust</b> and <b>Python</b> implementation of the <b>.SPZ</b> file format (v3) and <b>CLI</b> tools.</div>
&nbsp;
<div align="center"><b>WIP</b></div>
&nbsp;
<p align="center">
	<a href="https://crates.io/crates/spz" target="_blank">
		<img alt="Crates.io Version" src="https://img.shields.io/crates/v/spz?style=for-the-badge&link=https%3A%2F%2Fcrates.io%2Fcrates%2Fspz">
	</a>
	<a href="https://docs.rs/spz" target="_blank">
		<img alt="docs.rs" src="https://img.shields.io/docsrs/spz?style=for-the-badge&label=docs.rs&link=docs.rs%2Fspz">
	</a>
	<a href="https://lib.rs/crates/spz" target="_blank">
		<img alt="lib.rs" src="https://img.shields.io/badge/spz-librs?style=for-the-badge&label=Lib.rs&link=https%3A%2F%2Flib.rs%2Fcrates%2Fspz">
	</a>
	<a href="https://github.com/Jackneill/spz/tags" target="_blank">
		<img alt="GitHub Tag" src="https://img.shields.io/github/v/tag/Jackneill/spz?style=for-the-badge">
	</a>
	<br>
	<a href="https://github.com/Jackneill/spz/actions" target="_blank">
		<img alt="GitHub CI" src="https://img.shields.io/github/check-runs/Jackneill/spz/main?	style=for-the-badge&label=CI%3Amain">
	</a>
	<a href="https://deps.rs/repo/github/Jackneill/spz" target="_blank">
		<img alt="Deps" src="https://img.shields.io/deps-rs/repo/github/Jackneill/spz?style=for-the-badge">
	</a>
	<a href="https://github.com/Jackneill/spz/commits/main/" target="_blank">
		<img alt="GitHub Last Commit" src="https://img.shields.io/github/last-commit/Jackneill/spz/main?style=for-the-badge">
	</a>
	<br>
	<a href="https://codspeed.io/Jackneill/spz" target="_blank">
		<img alt="CodSpeed" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fcodspeed.io%2Fbadge.json&style=for-the-badge" />
	</a>
	<a href="https://codecov.io/github/Jackneill/spz" target="_blank">
		<img alt="CodeCov" src="https://codecov.io/github/Jackneill/spz/graph/badge.svg?style=for-the-badge&token=10QLWY4MWG"/>
	</a>
	<a href="https://app.codacy.com/gh/Jackneill/spz/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade" target="_blank">
		<img alt="Codacy grade" src="https://img.shields.io/codacy/grade/07c0a96e3369423988ba06a2691695ea/main?label=CODE%20QUALITY&style=for-the-badge&link=https%3A%2F%2Fapp.codacy.com%2Fgh%2FJackneill%2Fspz%2Fdashboard%3Futm_source%3Dgh%26utm_medium%3Dreferral%26utm_content%3D%26utm_campaign%3DBadge_grade">
	</a>
	<br>
	<a href="./LICENSE-MIT" target="_blank">
		<img alt="Licenses" src="https://img.shields.io/crates/l/spz?style=for-the-badge&link=github.com%2FJackneill%2Fspz%23license-mit">
	</a>
	<a href="https://app.fossa.com/projects/git%2Bgithub.com%2FJackneill%2Fspz?ref=badge_shield&issueType=license" alt="FOSSA Status" target="_blank">
		<img alt="FOSSA Status" src="https://app.fossa.com/api/projects/git%2Bgithub.com%2FJackneill%2Fspz.svg?type=shield&issueType=license"/>
	</a>
	<a href="https://app.fossa.com/projects/git%2Bgithub.com%2FJackneill%2Fspz?ref=badge_shield&issueType=security" alt="FOSSA Status" target="_blank">
		<img alt="FOSSA Security" src="https://app.fossa.com/api/projects/git%2Bgithub.com%2FJackneill%2Fspz.svg?type=shield&issueType=security"/>
	</a>
	<br>
	<a href="https://github.com/Jackneill/spz" target="_blank">
		<img alt="Python Version from PEP 621 TOML" src="https://img.shields.io/python/required-version-toml?tomlFilePath=https%3A%2F%2Fraw.githubusercontent.com%2FJackneill%2Fspz%2Frefs%2Fheads%2Fmain%2Fcrates%2Fspz-pywrapper%2Fpyproject.toml&style=for-the-badge&link=https%3A%2F%2Fgithub.com%2FJackneill%2Fspz">
	</a>
	<a href="https://pypi.org/project/spz/" target="_blank">
		<img alt="PyPI - Wheel" src="https://img.shields.io/pypi/wheel/spz?style=for-the-badge&link=https%3A%2F%2Fpypi.org%2Fproject%2Fspz%2F">
	</a>
	<hr>
	<br>
	<a href="https://flathub.org/apps/io.github.jackneill.spz" target="_blank">
		<img width="160" alt="Get it on Flathub" src="https://flathub.org/api/badge?locale=en"/>
	</a>
	<a href="https://snapcraft.io/spz" target="_blank">
		<img width="160" alt="Get it on Snapcraft Store" src="./assets/images/snap_badge.svg"/>
	</a>
</p>

## What is SPZ?

SPZ is a compressed file format for 3D Gaussian Splats, designed by Niantic.
It provides efficient storage of Gaussian Splat data with configurable
spherical harmonics degrees and coordinate system support.

See [docs/SPZ.md](docs/SPZ.md) for more information.

## CLI

```sh
$ # install:
$ cargo install spz
$ # or
$ flatpak install io.github.jackneill.spz
$ # or
$ snap install spz
$
$ # run:
$ path/to/spz info assets/racoonfamily.spz

GaussianSplat={num_points=932560, sh_degree=3, antialiased=true, median_ellipsoid_volume=0.0000000046213082, bbox=[x=-281.779541 to 258.382568, y=-240.000000 to 240.000000, z=-240.000000 to 240.000000]}
```

## Rust

## Usage

```toml
[dependencies]
spz = { version = "0.0.7", default-features = false, features = [] }
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

use anyhow::{Context, Result};
use spz::{coord::CoordinateSystem, gaussian_splat::GaussianSplat, unpacked::UnpackOptions};

fn main() -> Result<()> {
	let mut sample_spz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	sample_spz.push("../../assets/racoonfamily.spz");

	let _gs1 = GaussianSplat::builder()
		.filepath(&sample_spz)
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(CoordinateSystem::UNSPECIFIED)
				.build(),
		)
		.load()
		.with_context(|| format!("unable to load file: {:?}", sample_spz))?;

	let _gs0 = GaussianSplat::builder().filepath(&sample_spz).load()?;

	Ok(())
}

#[allow(unused)]
async fn load_spz_async<P>(spz_file: P) -> Result<GaussianSplat>
where
	P: AsRef<Path>,
{
	GaussianSplat::builder()
		.filepath(spz_file.as_ref())
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(CoordinateSystem::UNSPECIFIED)
				.build(),
		)
		.load_async()
		.await
		.with_context(|| format!("unable to load file: {:?}", spz_file.as_ref()))
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
* For fuzz testing: `cargo install cargo-fuzz`
 	* Further documentation is available in [fuzz/README.md](./fuzz/README.md).
* [Install cargo-mutants](https://mutants.rs/getting-started.html) for test insights.
 	* `cargo install cargo-mutants`

### Run

```sh
just test
just fuzz
just mutants
```

## Benches

### Pre-Requisites

* `cargo install cargo-criterion`
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
splat = spz.load("scene.spz")  # -> GaussianSplat
# or
splat = spz.GaussianSplat.load(
    "scene.spz", coordinate_system=spz.CoordinateSystem.RUB
)  # -> GaussianSplat
# or
with spz.SplatReader("scene.spz") as ctx:
    splat2 = ctx.splat  # -> GaussianSplat

with spz.temp_save(splat) as tmp_path:
    import subprocess

    subprocess.run(["viewer", str(tmp_path)])

# Access properties
print(f"{splat.num_points:,} points")
print(f"center: {splat.bbox.center}")
print(f"size: {splat.bbox.size}")

# Access data (flat arrays, list[float])
positions = splat.positions  # [x1, y1, z1, x2, y2, z2, ...]
scales = splat.scales
rotations = splat.rotations
alphas = splat.alphas
colors = splat.colors
sh = splat.spherical_harmonics

# Serialize
data = splat.to_bytes()  # -> bytes
splat2 = spz.GaussianSplat.from_bytes(data)  # -> GaussianSplat

# Create from data
new_splat = spz.GaussianSplat(
    positions=[0.0, 0.0, 0.0, 1.0, 2.0, 3.0],  # flat array
    scales=[-5.0] * 6,
    rotations=[1.0, 0.0, 0.0, 0.0] * 2,
    alphas=[0.5, 0.8],
    colors=[255.0, 0.0, 0.0, 0.0, 255.0, 0.0],
)  # -> GaussianSplat

# Save to file
new_splat.save("output.spz")

with spz.SplatWriter("output2.spz") as writer:
    writer.splat = splat2

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
