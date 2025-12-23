// SPDX-License-Identifier: Apache-2.0 OR MIT

use codspeed_criterion_compat::Criterion;

use crate::util;

fn print_info() {
	let gs = util::load_packed_from_file().unwrap();

	println!("{}", gs);
}

pub fn bench_print_info(c: &mut Criterion) {
	c.bench_function("print_info", |b| b.iter(|| print_info()));
}
