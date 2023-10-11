#![cfg_attr(feature = "used_linker", feature(used_with_arg))]

use linkme::distributed_slice;

#[distributed_slice]
static ITEMS: [&'static str];

#[distributed_slice(ITEMS)]
static ITEM1: &'static str = "item1";

// Regression test for https://github.com/dtolnay/linkme/issues/67.
//
// Must be run with `--release`.
#[test]
fn win_status_access_violation() {
    let mut last_address = None;
    for item in ITEMS {
        // Do some busy work to push the compiler into optimizing the code in a
        // particularly "bad" way. This is derived from experimentation.
        let address = item as *const &str as usize;
        if let Some(last) = last_address {
            assert_eq!(address - last, std::mem::size_of::<&str>());
        }
        last_address = Some(address);

        // Should not cause STATUS_ACCESS_VIOLATION.
        println!("{} {:?}", item.len(), item.as_bytes());
    }
}
