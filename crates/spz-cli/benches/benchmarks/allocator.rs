// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Shared benchmark workloads for comparing allocators.
//!
//! These benchmarks are invoked from two separate harnesses.
//!
//! Run both and compare the output to see the allocator effect.

use std::fmt::Write as _;
use std::path::Path;

use criterion::Criterion;
use spz::coord::CoordinateSystem;
use spz::gaussian_splat::GaussianSplat;
use spz::header::Header;

/// Path to test asset, relative to the bench working directory
/// (which is the crate root: `crates/spz-cli/`).
const ASSET_PATH: &str = "../../assets/racoonfamily.spz";

/// Simulates the `metainfo` command: parse header only.
fn workload_metainfo<P>(path: P)
where
	P: AsRef<Path>,
{
	let hdr = Header::from_file(path).expect("failed to read header");
	let mut buf = String::new();

	write!(buf, "{}", hdr.pretty_fmt()).unwrap();

	std::hint::black_box(buf);
}

/// Simulates the `info` command: full load.
fn workload_info<P>(path: P)
where
	P: AsRef<Path>,
{
	let gs = GaussianSplat::load(path).expect("failed to load SPZ");
	let mut buf = String::new();

	write!(buf, "{}", gs.pretty_fmt()).unwrap();

	std::hint::black_box(buf);
}

/// Pure load workload — heaviest allocation path (decompress + populate Vecs).
fn workload_load<P>(path: P)
where
	P: AsRef<Path>,
{
	let gs = GaussianSplat::builder()
		.packed(true)
		.expect("packed flag")
		.coord_sys(CoordinateSystem::default())
		.load(path)
		.expect("failed to load SPZ");

	std::hint::black_box(gs);
}

pub fn bench_metainfo(c: &mut Criterion) {
	let path = Path::new(ASSET_PATH);

	assert!(path.exists(), "test asset not found: {ASSET_PATH}");

	c.bench_function("metainfo", |b| b.iter(|| workload_metainfo(path)));
}

pub fn bench_info(c: &mut Criterion) {
	let path = Path::new(ASSET_PATH);

	assert!(path.exists(), "test asset not found: {ASSET_PATH}");

	c.bench_function("info", |b| b.iter(|| workload_info(path)));
}

pub fn bench_load(c: &mut Criterion) {
	let path = Path::new(ASSET_PATH);

	assert!(path.exists(), "test asset not found: {ASSET_PATH}");

	c.bench_function("load", |b| b.iter(|| workload_load(path)));
}
