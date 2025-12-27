// SPDX-License-Identifier: Apache-2.0 OR MIT

use codspeed_criterion_compat::Criterion;
use spz::{packed::PackOptions, prelude::GaussianSplat, unpacked::UnpackOptions};
use tokio::runtime::Runtime;

use crate::util;

pub fn bench_cloud_load_n(c: &mut Criterion) {
	let gs = util::create_splat(50_000);
	let temp_dir = std::env::temp_dir();
	let spz_path = temp_dir.join("large_cloud_performance.spz");

	gs.save_as_packed(&spz_path, &PackOptions::default())
		.unwrap();

	c.bench_function("splat_load_50_000_pts", |b| {
		b.iter(|| {
			GaussianSplat::load_packed_from_file(&spz_path, &UnpackOptions::default())
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
