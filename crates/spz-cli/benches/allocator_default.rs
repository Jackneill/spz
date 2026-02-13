// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark harness using the **default system allocator**.
//!
//! Compare results against `allocator_bench` (which uses mimalloc) to measure
//! the effect of mimalloc on spz-cli workloads.

use criterion::{criterion_group, criterion_main};

mod benchmarks;

criterion_group! {
	benches,
	benchmarks::allocator::bench_metainfo,
	benchmarks::allocator::bench_info,
	benchmarks::allocator::bench_load,
}
criterion_main!(benches);
