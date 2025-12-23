// SPDX-License-Identifier: Apache-2.0 OR MIT

use codspeed_criterion_compat::{criterion_group, criterion_main};

mod benchmarks;
mod util;

criterion_group! {
	benches,
	benchmarks::load::bench_load_packed_from_file,
	benchmarks::load::bench_cloud_load_n,
	benchmarks::save::bench_cloud_save_n,
	benchmarks::print_info::bench_print_info,
}
criterion_main!(benches);
