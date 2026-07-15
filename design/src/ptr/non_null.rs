use std::ptr::NonNull;

use crate::{
    ops::place::{
        PlaceHandle,
        ProjectPlace,
        ProxyPlace,
        borrowck::{
            AccessKind,
            Instant,
        },
    },
    place::Subplace,
};

impl<T: ?Sized> ProxyPlace for NonNull<T> {
    type Handle = Self;

    const ACCESS: AccessKind = AccessKind::Untracked;
    type Timing = Instant;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        unsafe { *this }
    }
}

impl<T: ?Sized> PlaceHandle for NonNull<T> {
    type Target = T;
}

unsafe impl<S: Subplace> ProjectPlace<S> for NonNull<S::Source> {
    type Projected = NonNull<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let ptr: *mut S::Source = self.as_ptr();
        let ptr: *mut S::Target = unsafe { ptr.project_place(subplace) };
        unsafe { NonNull::new_unchecked(ptr) }
    }
}
