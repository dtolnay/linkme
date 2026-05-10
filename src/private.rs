#[cfg(target_arch = "wasm32")]
use alloc::vec::Vec;
#[doc(hidden)]
pub use core::primitive::isize;
#[doc(hidden)]
pub use core::ptr;
#[cfg(target_arch = "wasm32")]
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};

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

/// A node in the intrusive linked list used to collect elements on wasm32.
///
/// Each registered element has exactly one `WasmEntryNode` stored in a static,
/// so registration never allocates.
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub struct WasmEntryNode<T> {
    sort_key: &'static str,
    element: *const T,
    next: AtomicPtr<WasmEntryNode<T>>,
}

#[cfg(target_arch = "wasm32")]
unsafe impl<T> Send for WasmEntryNode<T> {}

#[cfg(target_arch = "wasm32")]
unsafe impl<T> Sync for WasmEntryNode<T> {}

#[cfg(target_arch = "wasm32")]
impl<T> WasmEntryNode<T> {
    #[doc(hidden)]
    pub const fn new(sort_key: &'static str, element: *const T) -> Self {
        WasmEntryNode {
            sort_key,
            element,
            next: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

/// Per-slice registry that collects element nodes via `.init_array` constructors
/// and materializes the slice on first access.
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub struct WasmRegistry<T> {
    head: AtomicPtr<WasmEntryNode<T>>,
    finalized: AtomicBool,
    sorted_ptr: AtomicPtr<*const WasmEntryNode<T>>,
    sorted_len: AtomicUsize,
    slice_ptr: AtomicPtr<T>,
    slice_len: AtomicUsize,
    finalize_lock: AtomicBool,
    materialize_lock: AtomicBool,
}

#[cfg(target_arch = "wasm32")]
unsafe impl<T> Sync for WasmRegistry<T> {}

#[cfg(target_arch = "wasm32")]
impl<T> WasmRegistry<T> {
    #[doc(hidden)]
    pub const fn new() -> Self {
        WasmRegistry {
            head: AtomicPtr::new(core::ptr::null_mut()),
            finalized: AtomicBool::new(false),
            sorted_ptr: AtomicPtr::new(core::ptr::null_mut()),
            sorted_len: AtomicUsize::new(0),
            slice_ptr: AtomicPtr::new(core::ptr::null_mut()),
            slice_len: AtomicUsize::new(0),
            finalize_lock: AtomicBool::new(false),
            materialize_lock: AtomicBool::new(false),
        }
    }

    /// Push `node` onto the head of the linked list.  Lock-free.
    ///
    /// # Safety
    ///
    /// Must be called before finalize() runs, i.e. during `.init_array`
    /// constructor execution.
    #[doc(hidden)]
    pub unsafe fn register(&self, node: &'static WasmEntryNode<T>) {
        debug_assert!(
            !self.finalized.load(Ordering::Acquire),
            "WasmRegistry::register called after finalization"
        );
        let node_ptr = node as *const WasmEntryNode<T> as *mut WasmEntryNode<T>;
        let mut current = self.head.load(Ordering::Relaxed);
        loop {
            node.next.store(current, Ordering::Relaxed);
            match self
                .head
                .compare_exchange_weak(current, node_ptr, Ordering::Release, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl<T: 'static> WasmRegistry<T> {
    fn finalize(&'static self) -> &'static [*const WasmEntryNode<T>] {
        if self.finalized.load(Ordering::Acquire) {
            let ptr = self.sorted_ptr.load(Ordering::Acquire) as *const *const WasmEntryNode<T>;
            let len = self.sorted_len.load(Ordering::Acquire);
            return unsafe { core::slice::from_raw_parts(ptr, len) };
        }

        while self
            .finalize_lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        if self.finalized.load(Ordering::Acquire) {
            self.finalize_lock.store(false, Ordering::Release);
            let ptr = self.sorted_ptr.load(Ordering::Acquire) as *const *const WasmEntryNode<T>;
            let len = self.sorted_len.load(Ordering::Acquire);
            return unsafe { core::slice::from_raw_parts(ptr, len) };
        }

        let mut nodes: Vec<*const WasmEntryNode<T>> = Vec::new();
        let mut current = self.head.load(Ordering::Acquire) as *const WasmEntryNode<T>;
        while !current.is_null() {
            nodes.push(current);
            current = unsafe { (*current).next.load(Ordering::Acquire) };
        }

        nodes.sort_by(|&a, &b| unsafe { (*a).sort_key.cmp((*b).sort_key) });

        let leaked = Vec::leak(nodes);
        let len = leaked.len();
        let ptr = leaked.as_mut_ptr();

        self.sorted_len.store(len, Ordering::Release);
        self.sorted_ptr.store(ptr, Ordering::Release);
        self.finalized.store(true, Ordering::Release);
        self.finalize_lock.store(false, Ordering::Release);

        unsafe { core::slice::from_raw_parts(ptr as *const *const WasmEntryNode<T>, len) }
    }

    /// Return a slice of all registered elements, byte-copied into a stable
    /// `'static` allocation.  Idempotent; subsequent calls return the same slice.
    ///
    /// # Safety
    ///
    /// Must be called after all `.init_array` constructors have run.
    #[doc(hidden)]
    pub unsafe fn static_slice(&'static self) -> &'static [T] {
        let existing = self.slice_ptr.load(Ordering::Acquire);
        if !existing.is_null() {
            let len = self.slice_len.load(Ordering::Acquire);
            return unsafe { core::slice::from_raw_parts(existing, len) };
        }

        let nodes = self.finalize();

        while self
            .materialize_lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }

        let existing = self.slice_ptr.load(Ordering::Acquire);
        if !existing.is_null() {
            self.materialize_lock.store(false, Ordering::Release);
            let len = self.slice_len.load(Ordering::Acquire);
            return unsafe { core::slice::from_raw_parts(existing, len) };
        }

        let mut collected: Vec<T> = Vec::with_capacity(nodes.len());
        for &node_ptr in nodes {
            let out = unsafe { collected.as_mut_ptr().add(collected.len()) };
            unsafe {
                ptr::copy_nonoverlapping((*node_ptr).element, out, 1);
                collected.set_len(collected.len() + 1);
            }
        }

        let slice = Vec::leak(collected);
        let data_ptr = slice.as_ptr() as *mut T;
        let len = slice.len();

        self.slice_len.store(len, Ordering::Release);
        self.slice_ptr.store(data_ptr, Ordering::Release);
        self.materialize_lock.store(false, Ordering::Release);

        slice
    }

    /// Iterate over references to the original element statics in sorted order.
    ///
    /// Unlike `static_slice()`, this preserves the address of each element as
    /// declared, which matters for elements containing interior mutability.
    ///
    /// # Safety
    ///
    /// Must be called after all `.init_array` constructors have run.
    #[doc(hidden)]
    pub unsafe fn iter_refs(&'static self) -> WasmRefIter<T> {
        let nodes = self.finalize();
        WasmRefIter {
            inner: nodes.iter(),
        }
    }
}

/// Iterator over address-identical references to registered wasm32 elements.
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
pub struct WasmRefIter<T: 'static> {
    inner: core::slice::Iter<'static, *const WasmEntryNode<T>>,
}

#[cfg(target_arch = "wasm32")]
impl<T: 'static> Iterator for WasmRefIter<T> {
    type Item = &'static T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|&node_ptr| unsafe { &*(*node_ptr).element })
    }
}
