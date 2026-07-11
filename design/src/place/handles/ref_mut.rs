use std::{
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    ops::place::{
        BorrowPlace,
        DerefHandle,
        DerefPlace,
        DropPlace,
        PlaceHandle,
        ProjectPlace,
        ProxyPlace,
        ReadPlace,
        WritePlace,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
        },
    },
    place::subplace::Subplace,
};

pub struct MutHandle<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _lt: PhantomData<&'a mut T>,
}

impl<T: ?Sized> Clone for MutHandle<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> Copy for MutHandle<'_, T> {}

impl<'a, T: ?Sized> MutHandle<'a, T> {
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _lt: PhantomData,
        }
    }
}

impl<'a, T: ?Sized> ProxyPlace for &'a mut T {
    type Handle = MutHandle<'a, T>;
}

impl<'a, T: ?Sized> PlaceHandle for MutHandle<'a, T> {
    type Target = T;
}

unsafe impl<T: ?Sized> DerefHandle for &mut T {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Instant;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let ptr: *const *mut T = this.cast::<*mut T>();
        let handle = unsafe { *ptr };
        MutHandle {
            ptr: unsafe { NonNull::new_unchecked(handle) },
            _lt: PhantomData,
        }
    }
}

unsafe impl<T> WritePlace for MutHandle<'_, T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.ptr.write(value) };
    }
}

unsafe impl<T> DropPlace for MutHandle<'_, T> {
    unsafe fn drop_place(self) {
        unsafe { self.ptr.drop_in_place() };
    }
}

unsafe impl<T> ReadPlace for MutHandle<'_, T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.ptr.read() }
    }
}

unsafe impl<'a, S: Subplace<Target: 'a>> ProjectPlace<S>
    for MutHandle<'a, S::Source>
{
    type Projected = MutHandle<'a, S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        MutHandle {
            ptr: unsafe { self.ptr.project_place(subplace) },
            _lt: PhantomData,
        }
    }
}

unsafe impl<'a, P: ?Sized> DerefPlace<P::Timing, Instant> for MutHandle<'a, P>
where
    P: DerefHandle,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn deref_place(self) -> <Self::Target as ProxyPlace>::Handle {
        unsafe { P::handle_from_raw(self.ptr.as_ptr()) }
    }
}

unsafe impl<'a, 'b, T> BorrowPlace<&'b mut T> for MutHandle<'a, T>
where
    'a: 'b,
{
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'b>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'b mut T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
}
