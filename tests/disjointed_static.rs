#![cfg_attr(feature = "used_linker", feature(used_with_arg))]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unknown_lints, non_local_definitions)] // FIXME

use linkme::disjointed_static;
use once_cell::sync::Lazy;

#[disjointed_static]
static SHENANIGANS: i32;

#[disjointed_static(SHENANIGANS)]
static N: i32 = 9;

#[test]
fn test() {
    assert_eq!(*SHENANIGANS, 9);
}

#[test]
fn test_non_copy() {
    pub struct NonCopy(pub i32);

    #[disjointed_static]
    static NONCOPY: NonCopy;

    #[disjointed_static(NONCOPY)]
    static ELEMENT: NonCopy = NonCopy(9);

    assert_eq!(NONCOPY.0, 9);
}

#[test]
fn test_interior_mutable() {
    #[disjointed_static]
    static MUTABLE: Lazy<i32>;

    #[disjointed_static(MUTABLE)]
    static ELEMENT: Lazy<i32> = Lazy::new(|| -1);

    assert_eq!(*ELEMENT, -1);
}

#[test]
fn test_elided_lifetime() {
    #[disjointed_static]
    pub static MYITEM: &str;

    #[disjointed_static(MYITEM)]
    static ELEMENT: &str = "...";

    assert_eq!(*MYITEM, "...");
}

#[test]
fn test_legacy_syntax() {
    // Rustc older than 1.43 requires an initializer expression.
    #[disjointed_static]
    pub static LEGACY: &str = ..;

    // Note: unlike disjointed_static, distributed_slice always *requires* that
    // a definition item exists when linking a binary, as otherwise the linker
    // will complain about the missing symbol.
    #[disjointed_static(LEGACY)]
    pub static ELEMENT: &str = "...";
}
