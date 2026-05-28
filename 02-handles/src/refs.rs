use std::{marker::PhantomData, ptr::NonNull};

use crate::{
    borrowck::{AccessKind, Instant, Lifetime},
    ops::{BorrowPlace, DerefHandle, DerefPlace, PlaceHandle, ProjectPlace, ProxyPlace},
    subplace::Subplace,
};

pub struct MutHandle<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _lt: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> ProxyPlace for &'a mut T {
    type Handle = MutHandle<'a, T>;
}

impl<'a, T: ?Sized> PlaceHandle for MutHandle<'a, T> {
    type Target = T;
}

impl<'a, T: ?Sized> DerefHandle for &'a mut T {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'a>;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        let ptr: *const *mut T = this.cast::<*mut T>();
        let handle = unsafe { *ptr };
        MutHandle {
            ptr: unsafe { NonNull::new_unchecked(handle) },
            _lt: PhantomData,
        }
    }
}

impl<'a, S: Subplace<Target: 'a>> ProjectPlace<S> for MutHandle<'a, S::Source> {
    type Projected = MutHandle<'a, S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        MutHandle {
            ptr: unsafe { self.ptr.project_place(subplace) },
            _lt: PhantomData,
        }
    }
}

impl<'a, P: ?Sized> DerefPlace<P::Timing, Instant> for MutHandle<'a, P>
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

impl<'a, 'b, T> BorrowPlace<&'b mut T> for MutHandle<'a, T>
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
