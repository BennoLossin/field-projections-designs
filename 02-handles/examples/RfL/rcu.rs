use std::{
    pin::{Pin, UnsafePinned},
    sync::atomic::{AtomicPtr, Ordering},
};

use pin_project::pin_project;

use crate::{bindings, mutex::InsideOfMutex};

#[pin_project]
pub struct Rcu<P: RcuPointer> {
    #[pin]
    ptr: UnsafePinned<AtomicPtr<P::Target>>,
}

pub struct RcuGuard(());

pub fn read_lock() -> RcuGuard {
    RcuGuard(())
}

pub trait RcuPointer: Sized {
    type Target;

    fn into_raw(this: Self) -> *mut Self::Target;

    unsafe fn from_raw(raw: *mut Self::Target) -> Self;
}

impl<P: RcuPointer> Rcu<P> {
    pub unsafe fn read_raw<'a>(this: *const Self, guard: &'a RcuGuard) -> &'a P::Target {
        let _ = guard;
        let ptr = UnsafePinned::raw_get(unsafe { &raw const (*this).ptr });
        let ptr = unsafe { ptr.as_ref_unchecked() };
        let ptr = ptr.load(Ordering::Relaxed);
        unsafe { ptr.as_ref_unchecked() }
    }

    pub fn read<'a>(&'a self, guard: &'a RcuGuard) -> &'a P::Target {
        let _ = guard;
        let ptr = self.ptr.get();
        let ptr = unsafe { ptr.as_ref_unchecked() };
        let ptr = ptr.load(Ordering::Relaxed);
        unsafe { ptr.as_ref_unchecked() }
    }

    pub fn read_exclusive(self: Pin<&mut Self>, /* replaced by &mut self + !Move */) -> &P::Target {
        let this = self.project();
        let ptr = this.ptr.get_mut_pinned();
        let ptr = unsafe { ptr.as_ref_unchecked() };
        let ptr = ptr.as_ptr();
        let ptr = unsafe { ptr.read() };
        unsafe { ptr.as_ref_unchecked() }
    }

    pub fn write(
        self: Pin<&mut Self>, /* replaced by &mut self + !Move */
        new: P,
    ) -> RcuOld<P> {
        let this = self.project();
        let ptr = this.ptr.get_mut_pinned();
        let r = unsafe { ptr.as_ref_unchecked() };
        let old = r.load(Ordering::Relaxed);
        let new = P::into_raw(new);
        r.store(new, Ordering::Relaxed);
        RcuOld(unsafe { P::from_raw(old) })
    }
}

// implemented for every lock that supports `Rcu`
impl<P: RcuPointer> InsideOfMutex<Rcu<P>> {
    pub fn read<'a>(&'a self, guard: &'a RcuGuard) -> &'a P::Target {
        unsafe { Rcu::read_raw(self.ptr.as_ptr(), guard) }
    }
}

pub struct RcuOld<P>(P);

impl<P> Drop for RcuOld<P> {
    fn drop(&mut self) {
        unsafe { bindings::synchronize_rcu() };
    }
}

impl<T> RcuPointer for Box<T> {
    type Target = T;

    fn into_raw(this: Self) -> *mut Self::Target {
        Box::into_raw(this)
    }

    unsafe fn from_raw(raw: *mut Self::Target) -> Self {
        unsafe { Box::from_raw(raw) }
    }
}
