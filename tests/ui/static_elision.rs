#![cfg_attr(feature = "used_linker", feature(used_with_arg))]

use linkme::distributed_slice;

#[distributed_slice]
pub static MYSLICE: [&str] = [..];

#[distributed_slice(MYSLICE)]
static ELEMENT: &str = "...";

fn main() {}
