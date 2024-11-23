#[doc(hidden)]
pub use core::assert;
#[doc(hidden)]
pub use core::mem;
#[doc(hidden)]
pub use core::primitive::usize;
#[doc(hidden)]
pub use core::ptr;

#[cfg(target_family = "wasm")]
#[doc(hidden)]
pub use std;

#[cfg(target_family = "wasm")]
#[doc(hidden)]
pub use once_cell;

#[doc(hidden)]
pub trait Slice {
    type Element;
}

impl<T> Slice for [T] {
    type Element = T;
}

#[doc(hidden)]
pub enum Void {}


#[doc(hidden)]
pub use crate::distributed_slice::StaticPtr;