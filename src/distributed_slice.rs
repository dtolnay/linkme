use core::mem;
use core::ops::Deref;
use core::slice;

use crate::private::Slice;

pub struct DistributedSlice<T: ?Sized + Slice> {
    start: StaticPtr<T::Element>,
    stop: StaticPtr<T::Element>,
}

struct StaticPtr<T> {
    ptr: *const T,
}

unsafe impl<T> Send for StaticPtr<T> {}

unsafe impl<T> Sync for StaticPtr<T> {}

impl<T> Copy for StaticPtr<T> {}

impl<T> Clone for StaticPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> DistributedSlice<[T]> {
    #[doc(hidden)]
    pub const unsafe fn private_new(start: *const T, stop: *const T) -> Self {
        DistributedSlice {
            start: StaticPtr { ptr: start },
            stop: StaticPtr { ptr: stop },
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn private_typecheck(self, element: T) {
        mem::forget(element);
    }
}

impl<T> DistributedSlice<[T]> {
    pub fn static_slice(self) -> &'static [T] {
        let stride = mem::size_of::<T>();

        if stride == 0 {
            // We could make this work by storing a 1-byte companion entry in a
            // different link_section and using that count as the len if anyone
            // requires this to work, but for now just:
            return &[];
        }

        let start = self.start.ptr;
        let stop = self.stop.ptr;
        let byte_offset = stop as usize - start as usize;
        let len = byte_offset / stride;
        unsafe { slice::from_raw_parts(start, len) }
    }
}

impl<T> Copy for DistributedSlice<[T]> {}

impl<T> Clone for DistributedSlice<[T]> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Deref for DistributedSlice<[T]> {
    type Target = [T];
    fn deref(&self) -> &'static Self::Target {
        self.static_slice()
    }
}

impl<T: 'static> IntoIterator for DistributedSlice<[T]> {
    type Item = &'static T;
    type IntoIter = slice::Iter<'static, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.static_slice().iter()
    }
}
