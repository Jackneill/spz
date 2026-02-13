// SPDX-License-Identifier: Apache-2.0 OR MIT

use codspeed_criterion_compat::Criterion;
use spz::{
	gaussian_splat::GaussianSplat, gaussian_splat::LoadOptions, gaussian_splat::SaveOptions,
};
use tokio::runtime::Runtime;

use crate::benchmarks::util;

pub fn bench_cloud_load_n(c: &mut Criterion) {
	let gs = util::create_splat(50_000);
	let mut spz_path = util::tmpdir().unwrap();
	spz_path.push("large_cloud_performance.spz");

	gs.save(&spz_path, &SaveOptions::default()).unwrap();

	c.bench_function("splat_load_50_000_pts", |b| {
		b.iter(|| {
			GaussianSplat::load_with(&spz_path, &LoadOptions::default())
				.expect("failed to load");
		});
	});
	let _ = std::fs::remove_file(&spz_path);
}

pub fn bench_load_packed_from_file(c: &mut Criterion) {
	let rt = Runtime::new().unwrap();

	c.bench_function("load_packed_from_file", |b| {
		b.iter(|| util::load_packed_from_file());
	});
	c.bench_function("load_packed_from_file_async", |b| {
		b.to_async(&rt).iter(|| util::load_packed_from_file_async());
	});
}
