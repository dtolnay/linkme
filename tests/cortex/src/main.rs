#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};

use linkme::distributed_slice;

#[distributed_slice]
static SHENANIGANS: [i32] = [..];

#[distributed_slice(SHENANIGANS)]
static N: i32 = 9;

#[distributed_slice(SHENANIGANS)]
static NN: i32 = 99;

#[distributed_slice(SHENANIGANS)]
static NNN: i32 = 999;

#[entry]
fn main() -> ! {
    assert_eq!(SHENANIGANS.len(), 3);

    let mut sum = 0;
    for n in SHENANIGANS {
        sum += n;
    }

    assert_eq!(sum, 9 + 99 + 999);

    #[distributed_slice]
    static EMPTY: [i32] = [..];

    assert!(EMPTY.is_empty());

    hprintln!("success!");

    // exit QEMU
    debug::exit(debug::EXIT_SUCCESS);

    loop {}
}
