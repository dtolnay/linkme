use crate::private::Slice;
use core::fmt::{self, Debug};
#[cfg(any(target_os = "uefi", target_os = "windows"))]
use core::hint;
use core::mem;
use core::num::NonZeroUsize;
use core::ops::Deref;
use core::slice;

/// Collection of static elements.
///
/// On most platforms they are gathered into a contiguous section of the binary
/// by the linker. On wasm32 targets, startup constructors register elements
/// and the slice is materialized on first access.
///
/// The implementation is based on `link_section` attributes and
/// platform-specific linker support. Except on wasm32, it does not involve
/// life-before-main or any other runtime initialization. On wasm32, startup
/// constructors are used to collect elements because wasm custom sections
/// cannot carry the relocations needed by arbitrary Rust statics.
///
/// On `wasm32-unknown-emscripten` and WASI targets the runtime invokes
/// `.init_array` constructors automatically before `main`. On
/// `wasm32-unknown-unknown` the embedder must call `__wasm_call_ctors`
/// explicitly; wasm-bindgen handles this transparently.
///
/// Duplicate slice detection is performed via the linker on supported native
/// targets and is skipped on wasm32.
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
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct DistributedSlice<T: ?Sized + Slice> {
    name: &'static str,
    stride: NonZeroUsize,
    section_start: StaticPtr<T::Element>,
    section_stop: StaticPtr<T::Element>,
    dupcheck_start: StaticPtr<isize>,
    dupcheck_stop: StaticPtr<isize>,
    #[cfg(target_arch = "wasm32")]
    wasm_registry: StaticPtr<crate::private::WasmRegistry<T::Element>>,
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
    #[track_caller]
    pub const unsafe fn private_new(
        name: &'static str,
        section_start: *const T,
        section_stop: *const T,
        dupcheck_start: *const isize,
        dupcheck_stop: *const isize,
    ) -> Self {
        let Some(stride) = NonZeroUsize::new(mem::size_of::<T>()) else {
            panic!("#[distributed_slice] requires that the slice element type has nonzero size");
        };

        DistributedSlice {
            name,
            stride,
            section_start: StaticPtr { ptr: section_start },
            section_stop: StaticPtr { ptr: section_stop },
            dupcheck_start: StaticPtr {
                ptr: dupcheck_start,
            },
            dupcheck_stop: StaticPtr { ptr: dupcheck_stop },
            #[cfg(target_arch = "wasm32")]
            wasm_registry: StaticPtr {
                ptr: core::ptr::null(),
            },
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[doc(hidden)]
    #[track_caller]
    pub const unsafe fn private_new_wasm(
        name: &'static str,
        registry: *const crate::private::WasmRegistry<T>,
    ) -> Self {
        let Some(stride) = NonZeroUsize::new(mem::size_of::<T>()) else {
            panic!("#[distributed_slice] requires that the slice element type has nonzero size");
        };

        DistributedSlice {
            name,
            stride,
            section_start: StaticPtr {
                ptr: core::ptr::null(),
            },
            section_stop: StaticPtr {
                ptr: core::ptr::null(),
            },
            dupcheck_start: StaticPtr {
                ptr: core::ptr::null(),
            },
            dupcheck_stop: StaticPtr {
                ptr: core::ptr::null(),
            },
            wasm_registry: StaticPtr { ptr: registry },
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn private_typecheck(self, get: fn() -> &'static T) {
        let _ = get;
    }

    #[cfg(target_arch = "wasm32")]
    #[doc(hidden)]
    pub unsafe fn private_wasm_register(self, node: &'static crate::private::WasmEntryNode<T>)
    where
        T: 'static,
    {
        unsafe {
            (*self.wasm_registry.ptr).register(node);
        }
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
    /// On wasm32 targets, the slice is materialized on the first call from
    /// elements registered by startup constructors, then cached for all
    /// subsequent calls. The returned slice is a byte copy of the original
    /// element statics; for elements with interior mutability or that require
    /// address identity, use [`iter_refs()`][Self::iter_refs] instead.
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
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { (*self.wasm_registry.ptr).static_slice() }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // On Windows/UEFI, boundary elements are non-ZST (MaybeUninit<T>
            // and isize) so dupcheck and slice boundary arithmetic must
            // account for their size.
            let skip = usize::from(cfg!(any(target_os = "uefi", target_os = "windows")));
            if self.dupcheck_start.ptr.wrapping_add(skip + 1) < self.dupcheck_stop.ptr {
                panic!("duplicate #[distributed_slice] with name \"{}\"", self.name);
            }

            let start = unsafe { self.section_start.ptr.add(skip) };
            let stop = self.section_stop.ptr;
            let byte_offset = stop as usize - start as usize;
            let len = byte_offset / self.stride;

            // On Windows, the implementation involves growing a &[T; 0] to
            // encompass elements that we have asked the linker to place
            // immediately after that location. The compiler sees this as going
            // "out of bounds" based on provenance, so we must conceal what is
            // going on.
            #[cfg(any(target_os = "uefi", target_os = "windows"))]
            let start = hint::black_box(start);

            unsafe { slice::from_raw_parts(start, len) }
        }
    }

    /// Iterate over references to all registered elements at their original
    /// static addresses, in declaration/sort-key order.
    ///
    /// On most platforms this is equivalent to `self.static_slice().iter()`,
    /// because the linker section already holds the original element storage.
    /// On wasm32 targets, `static_slice()` returns references into a fresh
    /// byte-copy of the registered elements; `iter_refs()` returns references
    /// to the original element statics, which matters when elements contain
    /// interior mutability (`AtomicUsize`, `UnsafeCell`, etc.) or when pointer
    /// identity with the declaring static must be preserved.
    pub fn iter_refs(self) -> impl Iterator<Item = &'static T>
    where
        T: 'static,
    {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { (*self.wasm_registry.ptr).iter_refs() }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.static_slice().iter()
        }
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
