use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::NonNull,
};

use field_projection_design::design::{
    ops::{PlaceWrapper, WrapPlace},
    subplace::{Subplace, TransmutedSubplace},
};

use crate::{bindings, opaque::Opaque};

pub struct Mutex<T> {
    value: UnsafeCell<T>,
    mutex: Opaque<bindings::mutex>,
}

impl<T> Mutex<T> {
    pub fn lock(&self) -> Pin<MutexGuard<'_, T>> {
        unsafe { bindings::mutex_lock(self.mutex.get()) };
        let guard = MutexGuard(self);
        unsafe { Pin::new_unchecked(guard) }
    }
}

pub struct InsideOfMutex<T> {
    pub(crate) ptr: NonNull<T>,
}

pub struct MutexGuard<'a, T>(&'a Mutex<T>);

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { bindings::mutex_unlock(self.0.mutex.get()) };
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.value.get() }
    }
}

impl<T> PlaceWrapper for Mutex<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for Mutex<S::Source>
where
    S: Subplace<Source: Sized, Target: Sized>,
{
    type Wrapped = TransmutedSubplace<S, Mutex<S::Source>, InsideOfMutex<S::Target>>;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}

impl<T> PlaceWrapper for InsideOfMutex<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for InsideOfMutex<S::Source>
where
    S: Subplace<Source: Sized, Target: Sized>,
{
    type Wrapped = TransmutedSubplace<S, InsideOfMutex<S::Source>, InsideOfMutex<S::Target>>;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}
