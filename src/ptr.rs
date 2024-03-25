pub(crate) struct StaticPtr<T> {
    pub(crate) ptr: *const T,
}

unsafe impl<T> Send for StaticPtr<T> {}

unsafe impl<T> Sync for StaticPtr<T> {}

impl<T> Copy for StaticPtr<T> {}

impl<T> Clone for StaticPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}
