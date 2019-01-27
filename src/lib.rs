#![no_std]

mod distributed_slice;

#[doc(hidden)]
pub mod private;

pub use linkme_impl::*;

pub use distributed_slice::DistributedSlice;
