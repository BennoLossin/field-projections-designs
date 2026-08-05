//! The owned reference type [`Own<'a, T>`] and its API.
//!
//! The primary uses for owned references are
//!
//! 1. Passing ownership of unsized values, like `dyn Trait` and slices.
//! 2. Avoiding moves while passing ownership of values, without relying on
//!    heap allocation.

#![deny(missing_docs)]

use std::{
    fmt::Debug,
    marker::{
        CoercePointee,
        PhantomData,
    },
    mem::ManuallyDrop,
    ops::{
        Deref,
        DerefMut,
    },
    ptr::NonNull,
};

use crate::{
    ops::place::{
        BorrowPlace,
        CreateHandle,
        DropHusk,
        DropPlace,
        MovePlace,
        PlaceHandle,
        PlaceProxy,
        ProjectPlace,
        ReadPlace,
        WritePlace,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
        },
    },
    place::Subplace,
};

/// An owned reference.
///
/// Ownership is transferred to an `Own<'a, T>` when it is created. When the
/// reference is dropped, so is the pointee.
#[derive(CoercePointee)]
#[repr(transparent)]
pub struct Own<'a, T: ?Sized>(NonNull<T>, PhantomData<&'a T>);

pub use crate::own;

/// Create an owned reference [`Own<'a, T>`] to the expression.
///
/// Note that this may move the value in memory. In practice it should not.
#[macro_export]
macro_rules! own {
    ($e:expr) => {{
        // There's no such thing as "forget_in_place", i.e. disarm destructors
        // of an existing value without moving it or destroying its stack slot.
        // In practice, `ManuallyDrop::new()` is marked `#[inline(always)]` and
        // it's a no-op in terms of layout, so we hope that the compiler is
        // smart enough to avoid moves in most cases.
        super let mut owned = ::core::mem::ManuallyDrop::new($e);
        super let ref_mut = &mut owned;
        $crate::own::Own::from_manually_drop(ref_mut)
    }};
}

impl<'a, T: ?Sized> Own<'a, T> {
    /// Create an `Own` from a raw pointer and transfer ownership to it.
    ///
    /// ### Safety
    ///
    /// `raw` must adhere to the same safety requirements as `Box<T>` and
    /// references; that is, `raw` must be aligned and non-null, it cannot be
    /// dangling, and it must point to a valid value.
    ///
    /// The place pointed to by `raw` must outlive `'a` and must not be aliased
    /// by any other pointer.
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        Self(unsafe { NonNull::new_unchecked(raw) }, PhantomData)
    }

    /// Create an `Own` from a `&mut ManuallyDrop` and transfer ownership to it.
    /// The value will be dropped when the `Own` is dropped.
    pub fn from_manually_drop(md: &'a mut ManuallyDrop<T>) -> Self {
        Self(
            unsafe { NonNull::new_unchecked(&raw mut **md) },
            PhantomData,
        )
    }
}

impl<'a, T: ?Sized> Drop for Own<'a, T> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(self.0.as_ptr());
        }
    }
}

impl<'a, T: ?Sized> Deref for Own<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<'a, T: ?Sized> DerefMut for Own<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl<'a, T: ?Sized> Debug for Own<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::fmt(&*self, f)
    }
}

impl<'a, T: ?Sized> PlaceProxy for Own<'a, T> {
    type Target = T;
}

unsafe impl<'a, T: ?Sized> CreateHandle<Instant> for Own<'a, T> {
    type Handle = OwnHandle<'a, T>;

    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        OwnHandle {
            ptr: unsafe { &*this }.0,
            _lt: PhantomData,
        }
    }
}

/// [`PlaceProxy`] handle for [`Own<'a, T>`]. This type should not be used directly.
pub struct OwnHandle<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _lt: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> PlaceHandle for OwnHandle<'a, T> {
    type Target = T;
}

unsafe impl<'a, T> WritePlace for OwnHandle<'a, T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    unsafe fn write_place(self, value: Self::Target) {
        unsafe { self.ptr.write(value) }
    }
}

unsafe impl<'a, T> ReadPlace for OwnHandle<'a, T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.ptr.read() }
    }
}

// The superpower of this reference type.
// Mutable references cannot do this because some other type claims ownership.
unsafe impl<'a, T> MovePlace for OwnHandle<'a, T> {}

unsafe impl<'a, T: ?Sized> DropPlace for OwnHandle<'a, T> {
    unsafe fn drop_place(mut self) {
        unsafe { core::ptr::drop_in_place(self.ptr.as_mut()) }
    }
}

unsafe impl<'a, T: ?Sized> DropHusk for Own<'a, T> {
    unsafe fn drop_husk(_this: *mut Self) {}
}

unsafe impl<'a, S: Subplace> ProjectPlace<S> for OwnHandle<'a, S::Source>
where
    S::Target: 'a,
{
    type Projected = OwnHandle<'a, S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        OwnHandle {
            ptr: unsafe { self.ptr.project_place(subplace) },
            _lt: PhantomData,
        }
    }
}

// Corollary to Deref
unsafe impl<'a, T> BorrowPlace<&'a T> for OwnHandle<'a, T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

// Corollary to DerefMut
unsafe impl<'a, T> BorrowPlace<&'a mut T> for OwnHandle<'a, T> {
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> &'a mut T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

// Corollary to DerefOwn, I guess?
unsafe impl<'a, T> BorrowPlace<Own<'a, T>> for OwnHandle<'a, T> {
    // FIXME
    const ACCESS: AccessKind = AccessKind::Exclusive;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> Own<'a, T> {
        Own(self.ptr, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Mutex,
        atomic::{
            AtomicIsize,
            Ordering::Relaxed,
        },
    };

    use super::*;
    static LOCK: Mutex<()> = Mutex::new(());
    static TOTAL: AtomicIsize = AtomicIsize::new(0);
    static LIVE: AtomicIsize = AtomicIsize::new(0);

    #[derive(Debug)]
    struct Checked {
        #[expect(unused)]
        id: isize,
        dropped: bool,
    }

    impl Checked {
        fn new() -> Self {
            LIVE.fetch_add(1, Relaxed);
            Checked {
                id: TOTAL.fetch_add(1, Relaxed),
                dropped: false,
            }
        }
    }
    impl Drop for Checked {
        fn drop(&mut self) {
            assert!(!self.dropped);
            self.dropped = true;
            LIVE.fetch_sub(1, Relaxed);
        }
    }

    fn foo(r: Own<'_, Checked>) {
        dbg!(r);
    }
    fn choose<'a, T>(x: Own<'a, T>, y: Own<'a, T>, c: bool) -> Own<'a, T> {
        if c { x } else { y }
    }

    #[test]
    fn demo() {
        let _guard = LOCK.lock().unwrap();
        let a = own!(Checked::new());
        let b = own!(Checked::new());
        let c = own!(Checked::new());
        foo(c);
        dbg!(choose(a, b, true));

        assert_eq!(LIVE.load(Relaxed), 0);
    }

    #[test]
    fn whole() {
        let x = Checked::new();
        foo(own!(x));
    }

    #[test]
    fn nested() {
        let _guard = LOCK.lock().unwrap();
        struct Outer(Middle);
        struct Middle(Inner, Checked);
        struct Inner(Checked, Checked);

        {
            let mut out = Outer(Middle(
                Inner(Checked::new(), Checked::new()),
                Checked::new(),
            ));
            assert_eq!(LIVE.load(Relaxed), 3);

            foo(own!(out.0.1));
            foo(own!(out.0.0.0));
            assert_eq!(LIVE.load(Relaxed), 1);

            out.0.1 = Checked::new();
            assert_eq!(LIVE.load(Relaxed), 2);
            dbg!(choose(own!(out.0.0.1), own!(out.0.1), true));
            assert_eq!(LIVE.load(Relaxed), 0);

            out.0.0.0 = Checked::new();
            assert_eq!(LIVE.load(Relaxed), 1);
        }
        assert_eq!(LIVE.load(Relaxed), 0);
    }

    #[test]
    fn coerce_unsized() {
        fn takes_unsized(x: Own<'_, dyn Debug>) {
            dbg!(x);
        }

        let _guard = LOCK.lock().unwrap();

        let own = own!(Checked::new());
        takes_unsized(own);
        assert_eq!(LIVE.load(Relaxed), 0);
    }

    trait Trait: Debug {
        fn foo(self: Own<'_, Self>) {
            dbg!(&self);
        }
    }
    impl Trait for () {}
    impl Trait for i32 {}
    impl Trait for Checked {}

    #[test]
    fn receiver() {
        let _guard = LOCK.lock().unwrap();
        own!(Checked::new()).foo();
        assert_eq!(LIVE.load(Relaxed), 0);
    }
}
