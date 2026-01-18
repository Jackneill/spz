// SPDX-License-Identifier: Apache-2.0 OR MIT

use codspeed_criterion_compat::Criterion;
use spz::gaussian_splat::SaveOptions;

use crate::util;

pub fn bench_cloud_save_n(c: &mut Criterion) {
	let gs = util::create_splat(50_000);
	let mut spz_path = util::tmpdir().unwrap();
	spz_path.push("large_cloud_performance.spz");

	c.bench_function("splat_save_50_000_pts", |b| {
		b.iter(|| {
			gs.save_as_packed(&spz_path, &SaveOptions::default())
				.unwrap();
		});
	});
	let _ = std::fs::remove_file(&spz_path);
}
