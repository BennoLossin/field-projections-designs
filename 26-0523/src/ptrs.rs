use std::ptr::{self, NonNull};

use crate::{
    Metadata,
    borrowck::{AccessKind, Instant},
    ops::{
        DerefPlace, DropHusk, DropPlace, HandleFromRaw, MovePlace, PlaceHandle, ProjectPlace,
        ProxyPlace, ReadMetadata, ReadPlace, WritePlace,
    },
    subplace::Subplace,
};

impl<T: ?Sized> ProxyPlace for *const T {
    type Handle = Self;
}

impl<T: ?Sized> PlaceHandle for *const T {
    type Target = T;
}

impl<T: ?Sized> HandleFromRaw for *const T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        unsafe { *this }
    }
}

impl<T> ReadPlace for *const T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    const SAFE: bool = false;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.read() }
    }
}

impl<T: ?Sized> ReadMetadata for *const T {
    fn metadata(self) -> Metadata<Self::Target> {
        ptr::metadata(self)
    }
}

impl<T> MovePlace for *const T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    const SAFE: bool = false;
}

impl<S: Subplace> ProjectPlace<S> for *const S::Source {
    type Projected = *const S::Target;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let meta: Metadata<S::Source> = self.metadata();
        let thin: *const () = self.cast();
        let (offset, meta) = subplace.offset(meta);
        let thin = unsafe { thin.byte_add(offset) };
        ptr::from_raw_parts(thin, meta)
    }
}

impl<P> DerefPlace<P::Timing, Instant> for *const P
where
    P: ?Sized + HandleFromRaw,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = AccessKind::Untracked;

    const SAFE: bool = false;

    unsafe fn deref_place(self) -> <Self::Target as ProxyPlace>::Handle {
        unsafe { P::handle_from_raw(self) }
    }
}

// ---

impl<T: ?Sized> ProxyPlace for *mut T {
    type Handle = Self;
}

impl<T: ?Sized> PlaceHandle for *mut T {
    type Target = T;
}

impl<T: ?Sized> HandleFromRaw for *mut T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        unsafe { *this }
    }
}

impl<T> ReadPlace for *mut T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    const SAFE: bool = false;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.read() }
    }
}

impl<T: ?Sized> ReadMetadata for *mut T {
    fn metadata(self) -> Metadata<Self::Target> {
        ptr::metadata(self)
    }
}

impl<T> MovePlace for *mut T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    const SAFE: bool = false;
}

impl<T> WritePlace for *mut T {
    const ACCESS: AccessKind = AccessKind::Untracked;
    const SAFE: bool = false;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.write(value) }
    }
}

impl<T> DropPlace for *mut T {
    unsafe fn drop_place(self) {
        unsafe { self.drop_in_place() }
    }
}

impl<T> DropHusk for *mut T {
    unsafe fn drop_husk(_: Self::Handle) {}
}

impl<S: Subplace> ProjectPlace<S> for *mut S::Source {
    type Projected = *mut S::Target;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let meta: Metadata<S::Source> = self.metadata();
        let thin: *mut () = self.cast();
        let (offset, meta) = subplace.offset(meta);
        let thin = unsafe { thin.byte_add(offset) };
        ptr::from_raw_parts_mut(thin, meta)
    }
}

impl<P> DerefPlace<P::Timing, Instant> for *mut P
where
    P: ?Sized + HandleFromRaw,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = AccessKind::Untracked;

    const SAFE: bool = false;

    unsafe fn deref_place(self) -> <Self::Target as ProxyPlace>::Handle {
        unsafe { P::handle_from_raw(self) }
    }
}

// ---

impl<T: ?Sized> ProxyPlace for NonNull<T> {
    type Handle = Self;
}

impl<T: ?Sized> PlaceHandle for NonNull<T> {
    type Target = T;
}

impl<T: ?Sized> HandleFromRaw for NonNull<T> {
    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        unsafe { *this }
    }
}

impl<S: Subplace> ProjectPlace<S> for NonNull<S::Source> {
    type Projected = NonNull<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let ptr: *mut S::Source = self.as_ptr();
        let ptr: *mut S::Target = unsafe { ptr.project_place(subplace) };
        unsafe { NonNull::new_unchecked(ptr) }
    }
}
