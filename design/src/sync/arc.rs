use std::{
    ptr::NonNull,
    sync::{Arc, atomic::AtomicUsize},
};

use crate::ops::place::{
    BorrowPlace, DerefHandle, PlaceHandle, ProxyPlace,
    borrowck::{AccessKind, Instant, Lifetime},
};

#[repr(C, align(2))]
struct ArcInner<T: ?Sized> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    data: T,
}

pub struct ArcHandle<T: ?Sized>(NonNull<ArcInner<T>>);

impl<T: ?Sized> ProxyPlace for Arc<T> {
    type Handle = ArcHandle<T>;
}

impl<T: ?Sized> DerefHandle for Arc<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let ptr: *const NonNull<ArcInner<T>> = this.cast();
        let ptr: NonNull<ArcInner<T>> = unsafe { ptr.read() };
        ArcHandle(ptr)
    }
}

impl<T: ?Sized> PlaceHandle for ArcHandle<T> {
    type Target = T;
}

impl<'a, T: ?Sized> BorrowPlace<&'a T> for ArcHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a T {
        let ptr = self.0.as_ptr();
        unsafe { &(*ptr).data }
    }
}
