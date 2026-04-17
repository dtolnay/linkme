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

#[cfg(any(target_os = "uefi", target_os = "windows"))]
#[doc(hidden)]
pub type BoundaryElement<T> = core::mem::MaybeUninit<<T as Slice>::Element>;
