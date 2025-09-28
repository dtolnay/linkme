#[doc(hidden)]
pub use core::mem;
use core::mem::ManuallyDrop;
#[doc(hidden)]
pub use core::primitive::isize;
#[doc(hidden)]
pub use core::ptr;

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
pub union Padding<T> {
    b: u8,
    #[allow(dead_code)]
    t: ManuallyDrop<T>,
}

impl<T> Padding<T> {
    #[doc(hidden)]
    pub const VALUE: Self = Padding { b: 0 };
}

#[doc(hidden)]
pub const fn padding<T>() -> usize {
    if mem::size_of::<T>() == 0 {
        1
    } else {
        0
    }
}
