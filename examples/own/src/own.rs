use std::{
    marker::PhantomData,
    ptr::NonNull,
};

use design::ops::place::{
    BorrowPlace,
    CreateHandle,
    DropPlace,
    MovePlace,
    PlaceHandle,
    PlaceProxy,
    ReadPlace,
    WritePlace,
    borrowck::{
        AccessKind,
        Lifetime,
    },
};

pub struct Own<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _lt: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> Drop for Own<'a, T> {
    fn drop(&mut self) {
        unsafe { self.ptr.drop_in_place() };
    }
}

impl<T: ?Sized> PlaceProxy for Own<'_, T> {
    type Target = T;
}

pub struct OwnHandle<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _lt: PhantomData<&'a mut T>,
}

impl<T: ?Sized> PlaceHandle for OwnHandle<'_, T> {
    type Target = T;
}

unsafe impl<'a: 'b, 'b, T: ?Sized> CreateHandle<Lifetime<'b>> for Own<'a, T> {
    type Handle = OwnHandle<'b, T>;
    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        OwnHandle {
            ptr: unsafe { (*this).ptr },
            _lt: PhantomData,
        }
    }
}

unsafe impl<T> WritePlace for OwnHandle<'_, T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.ptr.write(value) }
    }
}

unsafe impl<T> ReadPlace for OwnHandle<'_, T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.ptr.read() }
    }
}

unsafe impl<T> MovePlace for OwnHandle<'_, T> {}

unsafe impl<T> DropPlace for OwnHandle<'_, T> {
    unsafe fn drop_place(self) {
        unsafe { self.ptr.drop_in_place() };
    }
}

unsafe impl<'a, T: ?Sized> BorrowPlace<&'a T> for OwnHandle<'a, T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a T {
        unsafe { self.ptr.as_ref() }
    }
}

unsafe impl<'a, T: ?Sized> BorrowPlace<&'a mut T> for OwnHandle<'a, T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(mut self) -> &'a mut T {
        unsafe { self.ptr.as_mut() }
    }
}
