#![cfg_attr(feature = "used_linker", feature(used_with_arg))]
#![allow(unknown_lints, non_local_definitions)] // FIXME
#![deny(rust_2024_compatibility, unsafe_op_in_unsafe_fn)]

use linkme::distributed_slice;
use once_cell::sync::Lazy;

#[distributed_slice]
static SHENANIGANS: [i32];

#[distributed_slice(SHENANIGANS)]
static N: i32 = 9;

#[distributed_slice(SHENANIGANS)]
static NN: i32 = 99;

#[distributed_slice(SHENANIGANS)]
static NNN: i32 = 999;

#[test]
fn test() {
    assert_eq!(SHENANIGANS.len(), 3);

    let mut sum = 0;
    for n in SHENANIGANS {
        sum += n;
    }

    assert_eq!(sum, 9 + 99 + 999);
}

#[test]
fn test_empty() {
    #[distributed_slice]
    static EMPTY: [i32];

    assert!(EMPTY.is_empty());
}

#[test]
fn test_non_copy() {
    pub struct NonCopy(#[allow(dead_code)] pub i32);

    #[distributed_slice]
    static NONCOPY: [NonCopy];

    #[distributed_slice(NONCOPY)]
    static ELEMENT: NonCopy = NonCopy(9);

    assert!(!NONCOPY.is_empty());
}

#[test]
fn test_interior_mutable() {
    #[distributed_slice]
    static MUTABLE: [Lazy<i32>];

    #[distributed_slice(MUTABLE)]
    static ELEMENT: Lazy<i32> = Lazy::new(|| -1);

    assert!(MUTABLE.len() == 1);
    assert!(*MUTABLE[0] == -1);
}

#[test]
fn test_elided_lifetime() {
    #[distributed_slice]
    pub static MYSLICE: [&str];

    #[distributed_slice(MYSLICE)]
    static ELEMENT: &str = "...";

    assert!(!MYSLICE.is_empty());
    assert_eq!(MYSLICE[0], "...");
}

#[test]
fn test_legacy_syntax() {
    // Rustc older than 1.43 requires an initializer expression.
    #[distributed_slice]
    pub static LEGACY: [&str] = [..];
}
