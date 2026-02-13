// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark harness using **mimalloc** as the global allocator.
//!
//! Compare results against `allocator_default` (which uses the system
//! allocator) to measure the effect of mimalloc on spz-cli workloads.

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use criterion::{criterion_group, criterion_main};

mod benchmarks;

criterion_group! {
	benches,
	benchmarks::allocator::bench_metainfo,
	benchmarks::allocator::bench_info,
	benchmarks::allocator::bench_load,
}
criterion_main!(benches);
