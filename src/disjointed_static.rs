use core::ops::Deref;

use crate::ptr::StaticPtr;

/// A static element which is disjointedly declared in one location and defined
/// in a different location to be linked together.
///
/// The implementation is based on `link_name` and `export_name` attributes.
/// It does not involve life-before-main or any other runtime initialization on
/// any platform. This is a zero-cost safe abstraction that operates entirely
/// during compilation and linking.
///
/// Like [`DistributedSlice`], platform-specific linker support is used to
/// detect duplicated declarations. Unlike `DistributedSlice`, duplicated
/// definition results in multiple definition of a linker symbol. The Rust
/// compiler can sometimes spot this and emit a nice error message, but more
/// often improper use of `DisjointedStatic` will result in a linker error.
///
/// [`DistributedSlice`]: crate::DistributedSlice
///
/// # Declaration
///
/// A disjointed static may be declared by writing `#[disjointed_static]` on a
/// static item whose type is some type `T`.
///
/// ```no_run
/// # #![cfg_attr(feature = "used_linker", feature(used_with_arg))]
/// #
/// # struct Config;
/// #
/// use linkme::disjointed_static;
///
/// #[disjointed_static]
/// pub static DEFAULT_CONFIG: Config;
/// ```
///
/// The attribute rewrites the `T` type of the static into
/// `DisjointedStatic<T>`, so the static in the example technically has type
/// `DisjointedStatic<Config>`.
///
/// ## Definition
///
/// The item definition for a disjointed static may be registered by a
/// `#[disjointed_static(...)]` attribute in which the path to the disjointed
/// static declaration is given in the parentheses. The initializer is required
/// to be a const expression.
///
/// The item definition may be registered in the same crate that declares the
/// disjointed static, or in any downstream crate. The definition linked into
/// the final binary will be observed in the disjointed static at runtime.
///
/// ```
/// # #![cfg_attr(feature = "used_linker", feature(used_with_arg))]
/// #
/// # mod other_crate {
/// #     use linkme::disjointed_static;
/// #
/// #     pub struct Config;
/// #
/// #     #[disjointed_static]
/// #     pub static DEFAULT_CONFIG: Config;
/// # }
/// #
/// # use other_crate::Config;
/// #
/// use linkme::disjointed_static;
/// use other_crate::DEFAULT_CONFIG;
///
/// #[disjointed_static(DEFAULT_CONFIG)]
/// static CONFIG: Config = Config {
///     /* ... */
/// };
/// ```
///
/// The compiler will require that the static item type matches with the type
/// of the disjointed static declaration. If the two do not match, the program
/// will not compile.
///
/// ```compile_fail
/// # #![cfg_attr(feature = "used_linker", feature(used_with_arg))]
/// #
/// # mod other_crate {
/// #     use linkme::disjointed_static;
/// #
/// #     pub struct Config;
/// #
/// #     #[disjointed_static]
/// #     pub static DEFAULT_CONFIG: Config;
/// # }
/// #
/// # use linkme::disjointed_static;
/// # use other_crate::DEFAULT_CONFIG;
/// #
/// #[disjointed_static(DEFAULT_CONFIG)]
/// static CONFIG_WTF: usize = 999;
/// ```
///
/// ```text
/// error[E0308]: mismatched types
///   --> tests/ui/mismatched_types.rs
///    |
/// LL | #[disjointed_static(DEFAULT_CONFIG)]
///    | ------------------------------------ arguments to this function are incorrect
/// LL | static CONFIG_WTF: usize = 999;
///    |                    ^^^^^ expected `Config`, found `usize`
///    |
///    = note: expected fn pointer `fn() -> &'static Config`
///               found fn pointer `fn() -> &'static usize`
/// ```
pub struct DisjointedStatic<T> {
    name: &'static str,
    singleton: StaticPtr<T>,
    dupcheck_start: StaticPtr<usize>,
    dupcheck_stop: StaticPtr<usize>,
}

impl<T> DisjointedStatic<T> {
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
        target_os = "psp",
    ))]
    pub const unsafe fn private_new(
        name: &'static str,
        singleton: *const T,
        dupcheck_start: *const usize,
        dupcheck_stop: *const usize,
    ) -> Self {
        DisjointedStatic {
            name,
            singleton: StaticPtr { ptr: singleton },
            dupcheck_start: StaticPtr {
                ptr: dupcheck_start,
            },
            dupcheck_stop: StaticPtr { ptr: dupcheck_stop },
        }
    }

    #[doc(hidden)]
    #[cfg(target_os = "windows")]
    pub const unsafe fn private_new(
        name: &'static str,
        singleton: *const T,
        dupcheck_start: *const (),
        dupcheck_stop: *const (),
    ) -> Self {
        DisjointedStatic {
            name,
            singleton: StaticPtr { ptr: singleton },
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
    pub const fn private_typecheck(self, get: fn() -> &'static T) {
        let _ = get;
    }
}

impl<T> DisjointedStatic<T> {
    /// Retrieve a static reference to the linked static item.
    ///
    /// **Note**: Ordinarily this method should not need to be called because
    /// `DisjointedStatic<T>` already behaves like `&'static T` in most ways
    /// through the power of `Deref`. In particular, function and method calls
    /// can all happen directly on the static without calling `static_item()`.
    ///
    /// ```no_run
    /// # #![cfg_attr(feature = "used_linker", feature(used_with_arg))]
    /// #
    /// # trait Runtime: Sync {
    /// #     fn run(&self, f: Box<dyn FnOnce()>);
    /// # }
    /// #
    /// use linkme::disjointed_static;
    ///
    /// #[disjointed_static]
    /// static RUNTIME: &dyn Runtime;
    ///
    /// #[disjointed_static]
    /// static CALLBACK: fn();
    ///
    /// fn main() {
    ///     // Invoke methods on the linked static item.
    ///     RUNTIME.run(Box::new(|| { /* ... */ }));
    ///
    ///     // Directly call the linked function.
    ///     CALLBACK();
    /// }
    /// ```
    pub fn static_item(self) -> &'static T {
        if self.dupcheck_start.ptr.wrapping_add(1) < self.dupcheck_stop.ptr {
            panic!("duplicate #[disjointed_static] with name \"{}\"", self.name);
        }

        unsafe { &*self.singleton.ptr }
    }
}

impl<T> Copy for DisjointedStatic<T> {}

impl<T> Clone for DisjointedStatic<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Deref for DisjointedStatic<T> {
    type Target = T;
    fn deref(&self) -> &'static Self::Target {
        self.static_item()
    }
}
