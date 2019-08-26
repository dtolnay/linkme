use linkme::distributed_slice;

pub struct Bencher;

#[distributed_slice]
pub static BENCHMARKS: [fn(&mut Bencher)] = [..];

#[distributed_slice(BENCHMARKS)]
static BENCH_WTF: usize = 999;

fn main() {}
