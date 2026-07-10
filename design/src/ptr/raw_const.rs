use std::ptr;

use crate::{
    ops::place::{
        DerefHandle,
        DerefPlace,
        MovePlace,
        PlaceHandle,
        ProjectPlace,
        ProxyPlace,
        ReadMetadata,
        ReadPlace,
        borrowck::{
            AccessKind,
            Instant,
        },
        subplace::Subplace,
    },
    ptr::Metadata,
};

impl<T: ?Sized> ProxyPlace for *const T {
    type Handle = Self;
}

impl<T: ?Sized> PlaceHandle for *const T {
    type Target = T;
}

impl<T: ?Sized> DerefHandle for *const T {
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
    P: ?Sized + DerefHandle,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = AccessKind::Untracked;

    const SAFE: bool = false;

    unsafe fn deref_place(self) -> <Self::Target as ProxyPlace>::Handle {
        unsafe { P::handle_from_raw(self) }
    }
}
