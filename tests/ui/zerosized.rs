use linkme::distributed_slice;

pub struct Unit;

#[distributed_slice]
pub static ZEROSIZED: [Unit] = [..];

fn main() {}
