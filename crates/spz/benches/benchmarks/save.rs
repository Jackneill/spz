// SPDX-License-Identifier: Apache-2.0 OR MIT

use codspeed_criterion_compat::Criterion;
use spz::packed::PackOptions;

use crate::util;

pub fn bench_cloud_save_n(c: &mut Criterion) {
	let gs = util::create_splat(50_000);
	let temp_dir = std::env::temp_dir();
	let spz_path = temp_dir.join("large_cloud_performance.spz");

	c.bench_function("splat_save_50_000_pts", |b| {
		b.iter(|| {
			gs.save_as_packed(&spz_path, &PackOptions::default())
				.unwrap();
		});
	});
	let _ = std::fs::remove_file(&spz_path);
}
