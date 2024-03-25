#![cfg_attr(feature = "used_linker", feature(used_with_arg))]

use linkme::{disjointed_static, distributed_slice};

pub struct Bencher;
pub struct Config;

#[distributed_slice]
pub static BENCHMARKS: [fn(&mut Bencher)];

#[disjointed_static]
pub static DEFAULT_CONFIG: Config;

#[distributed_slice(BENCHMARKS)]
static BENCH_WTF: usize = 999;

#[disjointed_static(DEFAULT_CONFIG)]
static CONFIG_WTF: usize = 999;

#[distributed_slice(BENCHMARKS)]
fn wrong_bench_fn<'a>(_: &'a mut ()) {}

fn main() {}
