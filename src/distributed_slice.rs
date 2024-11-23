#[allow(unused_imports)]
use core::alloc::Layout;
#[allow(unused_imports)]
use core::cell::UnsafeCell;
use core::fmt::{self, Debug};
use core::hint;
use core::mem;
use core::ops::Deref;
use core::slice;

use crate::__private::Slice;

/// Collection of static elements that are gathered into a contiguous section of
/// the binary by the linker.
///
/// The implementation is based on `link_section` attributes and
/// platform-specific linker support. It does not involve life-before-main or
/// any other runtime initialization on any platform. This is a zero-cost safe
/// abstraction that operates entirely during compilation and linking.
///
/// ## Declaration
///
/// A static distributed slice may be declared by writing `#[distributed_slice]`
/// on a static item whose type is `[T]` for some type `T`.
///
/// ```
/// # #![cfg_attr(feature = "used_linker", feature(used_with_arg))]
/// #
/// # struct Bencher;
/// #
/// use linkme::distributed_slice;
///
/// #[distributed_slice]
/// pub static BENCHMARKS: [fn(&mut Bencher)];
/// ```
///
/// The attribute rewrites the `[T]` type of the static into
/// `DistributedSlice<[T]>`, so the static in the example technically has type
/// `DistributedSlice<[fn(&mut Bencher)]>`.
///
/// ## Elements
///
/// Slice elements may be registered into a distributed slice by a
/// `#[distributed_slice(...)]` attribute in which the path to the distributed
/// slice is given in the parentheses. The initializer is required to be a const
/// expression.
///
/// Elements may be defined in the same crate that declares the distributed
/// slice, or in any downstream crate. Elements across all crates linked into
/// the final binary will be observed to be present in the slice at runtime.
///
/// ```
/// # #![cfg_attr(feature = "used_linker", feature(used_with_arg))]
/// #
/// # mod other_crate {
/// #     use linkme::distributed_slice;
/// #
/// #     pub struct Bencher;
/// #
/// #     #[distributed_slice]
/// #     pub static BENCHMARKS: [fn(&mut Bencher)];
/// # }
/// #
/// # use other_crate::Bencher;
/// #
/// use linkme::distributed_slice;
/// use other_crate::BENCHMARKS;
///
/// #[distributed_slice(BENCHMARKS)]
/// static BENCH_DESERIALIZE: fn(&mut Bencher) = bench_deserialize;
///
/// fn bench_deserialize(b: &mut Bencher) {
///     /* ... */
/// }
/// ```
///
/// The compiler will require that the static element type matches with the
/// element type of the distributed slice. If the two do not match, the program
/// will not compile.
///
/// ```compile_fail
/// # mod other_crate {
/// #     use linkme::distributed_slice;
/// #
/// #     pub struct Bencher;
/// #
/// #     #[distributed_slice]
/// #     pub static BENCHMARKS: [fn(&mut Bencher)];
/// # }
/// #
/// # use linkme::distributed_slice;
/// # use other_crate::BENCHMARKS;
/// #
/// #[distributed_slice(BENCHMARKS)]
/// static BENCH_WTF: usize = 999;
/// ```
///
/// ```text
/// error[E0308]: mismatched types
///   --> src/distributed_slice.rs:65:19
///    |
/// 17 | static BENCH_WTF: usize = 999;
///    |                   ^^^^^ expected fn pointer, found `usize`
///    |
///    = note: expected fn pointer `fn(&mut other_crate::Bencher)`
///                     found type `usize`
/// ```
///
/// ## Function elements
///
/// As a shorthand for the common case of distributed slices containing function
/// pointers, the distributed\_slice attribute may be applied directly to a
/// function definition to place a pointer to that function into a distributed
/// slice.
///
/// ```
/// # #![cfg_attr(feature = "used_linker", feature(used_with_arg))]
/// #
/// # pub struct Bencher;
/// #
/// use linkme::distributed_slice;
///
/// #[distributed_slice]
/// pub static BENCHMARKS: [fn(&mut Bencher)];
///
/// // Equivalent to:
/// //
/// //    #[distributed_slice(BENCHMARKS)]
/// //    static _: fn(&mut Bencher) = bench_deserialize;
/// //
/// #[distributed_slice(BENCHMARKS)]
/// fn bench_deserialize(b: &mut Bencher) {
///     /* ... */
/// }
/// ```
pub struct DistributedSlice<T: ?Sized + Slice> {
    name: &'static str,
    #[cfg(not(target_family = "wasm"))]
    section_start: StaticPtr<T::Element>,
    #[cfg(not(target_family = "wasm"))]
    section_stop: StaticPtr<T::Element>,
    #[cfg(not(target_family = "wasm"))]
    dupcheck_start: StaticPtr<usize>,
    #[cfg(not(target_family = "wasm"))]
    dupcheck_stop: StaticPtr<usize>,
    #[cfg(target_family = "wasm")]
    lazy: &'static ::std::sync::OnceLock<StaticPtr<T::Element>>,
    #[cfg(target_family = "wasm")]
    init: unsafe fn(*mut T::Element) -> *mut T::Element,
    #[cfg(target_family = "wasm")]
    len: unsafe fn() -> usize,
}
#[doc(hidden)]
pub struct StaticPtr<T> {
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
    #[cfg(target_family = "wasm")]
    pub const unsafe fn private_new(
        name: &'static str,
        init: unsafe fn(*mut T::Element) -> *mut T::Element,
        len: unsafe fn() -> usize,
        lazy: &'static ::std::sync::OnceLock<StaticPtr<T::Element>>,
    ) -> Self {
        DistributedSlice {
            name,
            init,
            len,
            lazy,
        }
    }
    #[doc(hidden)]
    #[cfg(any(
        target_os = "none",
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "psp",
    ))]
    pub const unsafe fn private_new(
        name: &'static str,
        section_start: *const T,
        section_stop: *const T,
        dupcheck_start: *const usize,
        dupcheck_stop: *const usize,
    ) -> Self {
        DistributedSlice {
            name,
            section_start: StaticPtr { ptr: section_start },
            section_stop: StaticPtr { ptr: section_stop },
            dupcheck_start: StaticPtr {
                ptr: dupcheck_start,
            },
            dupcheck_stop: StaticPtr { ptr: dupcheck_stop },
        }
    }

    #[doc(hidden)]
    #[cfg(any(target_os = "uefi", target_os = "windows"))]
    pub const unsafe fn private_new(
        name: &'static str,
        section_start: *const [T; 0],
        section_stop: *const [T; 0],
        dupcheck_start: *const (),
        dupcheck_stop: *const (),
    ) -> Self {
        DistributedSlice {
            name,
            section_start: StaticPtr {
                ptr: section_start as *const T,
            },
            section_stop: StaticPtr {
                ptr: section_stop as *const T,
            },
            dupcheck_start: StaticPtr {
                ptr: dupcheck_start as *const usize,
            },
            dupcheck_stop: StaticPtr {
                ptr: dupcheck_stop as *const usize,
            },
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn private_typecheck(self, get: fn() -> &'static T) {
        let _ = get;
    }

    /// Retrieve a contiguous slice containing all the elements linked into this
    /// program.
    ///
    /// **Note**: Ordinarily this method should not need to be called because
    /// `DistributedSlice<[T]>` already behaves like `&'static [T]` in most ways
    /// through the power of `Deref`. In particular, iteration and indexing and
    /// method calls can all happen directly on the static without calling
    /// `static_slice()`.
    ///
    /// ```no_run
    /// # #![cfg_attr(feature = "used_linker", feature(used_with_arg))]
    /// #
    /// # struct Bencher;
    /// #
    /// use linkme::distributed_slice;
    ///
    /// #[distributed_slice]
    /// static BENCHMARKS: [fn(&mut Bencher)];
    ///
    /// fn main() {
    ///     // Iterate the elements.
    ///     for bench in BENCHMARKS {
    ///         /* ... */
    ///     }
    ///
    ///     // Index into the elements.
    ///     let first = BENCHMARKS[0];
    ///
    ///     // Slice the elements.
    ///     let except_first = &BENCHMARKS[1..];
    ///
    ///     // Invoke methods on the underlying slice.
    ///     let len = BENCHMARKS.len();
    /// }
    /// ```
    pub fn static_slice(self) -> &'static [T] {
        #[cfg(not(target_family = "wasm"))]
        {
            if self.dupcheck_start.ptr.wrapping_add(1) < self.dupcheck_stop.ptr {
                panic!("duplicate #[distributed_slice] with name \"{}\"", self.name);
            }

            let stride = mem::size_of::<T>();
            let start = self.section_start.ptr;
            let stop = self.section_stop.ptr;
            let byte_offset = stop as usize - start as usize;
            let len = match byte_offset.checked_div(stride) {
                Some(len) => len,
                // The #[distributed_slice] call checks `size_of::<T>() > 0` before
                // using the unsafe `private_new`.
                None => unsafe { hint::unreachable_unchecked() },
            };

            // On Windows, the implementation involves growing a &[T; 0] to
            // encompass elements that we have asked the linker to place immediately
            // after that location. The compiler sees this as going "out of bounds"
            // based on provenance, so we must conceal what is going on.
            #[cfg(any(target_os = "uefi", target_os = "windows"))]
            let start = hint::black_box(start);

            return unsafe { slice::from_raw_parts(start, len) };
        };

        #[cfg(target_family = "wasm")]
        {
            let len = unsafe { (self.len)() };
            let start = *self.lazy.get_or_init(|| {
                let start = unsafe { ::std::alloc::alloc(Layout::array::<T::Element>(len)) }
                    as *mut T::Element;
                let end = unsafe { (self.init)(start) };
                if end != start.offset(len) {
                    panic!("invariant violated: `init` must fill the whole array")
                }
                return start;
            });
            return unsafe { slice::from_raw_parts(start, len) };
        };
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

impl<T> Debug for DistributedSlice<[T]>
where
    T: Debug + 'static,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.static_slice(), formatter)
    }
}
