// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::Result;
use criterion::{Criterion, criterion_group, criterion_main};
use spz::{GaussianSplat, UnpackOptions};
use tokio::runtime::Runtime;

fn bench_load_packed_from_file(c: &mut Criterion) {
	let rt = Runtime::new().unwrap();

	c.bench_function("load_packed_from_file", |b| {
		b.iter(|| load_packed_from_file())
	});
	c.bench_function("load_packed_from_file_async", |b| {
		b.to_async(&rt).iter(|| load_packed_from_file_async());
	});
	c.bench_function("print_info", |b| b.iter(|| print_info()));
}

fn load_packed_from_file() -> Result<GaussianSplat> {
	spz::GaussianSplat::builder()
		.filepath("assets/racoonfamily.spz")
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(spz::CoordinateSystem::default())
				.build(),
		)
		.load()
}

async fn load_packed_from_file_async() -> Result<GaussianSplat> {
	spz::GaussianSplat::builder()
		.filepath("assets/racoonfamily.spz")
		.packed(true)?
		.unpack_options(
			UnpackOptions::builder()
				.to_coord_system(spz::CoordinateSystem::default())
				.build(),
		)
		.load_async()
		.await
}

fn print_info() {
	let gs = load_packed_from_file().unwrap();

	println!("{}", gs);
}

criterion_group!(benches, bench_load_packed_from_file);
criterion_main!(benches);
