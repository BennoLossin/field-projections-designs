use crate::design::{
    borrowck::{AccessKind, Instant},
    ops::{DerefHandle, DerefPlace, PlaceHandle, ProxyPlace},
};

pub struct LocalHandle<T: ?Sized> {
    ptr: *const T,
}

impl<T: ?Sized> LocalHandle<T> {
    pub unsafe fn new(ptr: *const T) -> Self {
        Self { ptr }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.cast_mut()
    }
}

impl<T: ?Sized> PlaceHandle for LocalHandle<T> {
    type Target = T;
}

impl<P> DerefPlace<P::Timing, Instant> for LocalHandle<P>
where
    P: DerefHandle,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = P::ACCESS;

    const SAFE: bool = true;

    unsafe fn deref_place(self) -> <Self::Target as ProxyPlace>::Handle {
        unsafe { P::handle_from_raw(self.as_ptr()) }
    }
}
