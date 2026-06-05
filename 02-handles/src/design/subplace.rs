use std::{marker::PhantomData, ptr::Pointee};

use crate::design::Metadata;

pub unsafe trait Subplace: Sized {
    type Source: ?Sized;
    type Target: ?Sized;

    fn offset(self, metadata: Metadata<Self::Source>) -> (usize, Metadata<Self::Target>);
}

#[cfg(feature = "place-wrappers")]
pub struct TransmutedSubplace<P, S: ?Sized, T: ?Sized>(P, PhantomData<S>, PhantomData<T>);

#[cfg(feature = "place-wrappers")]
unsafe impl<P, S, T> Subplace for TransmutedSubplace<P, S, T>
where
    P: Subplace,
    S: ?Sized + Pointee<Metadata = Metadata<P::Source>>,
    T: ?Sized + Pointee<Metadata = Metadata<P::Target>>,
{
    type Source = S;

    type Target = T;
    fn offset(self, metadata: Metadata<Self::Source>) -> (usize, Metadata<Self::Target>) {
        self.0.offset(metadata)
    }
}

#[cfg(feature = "place-wrappers")]
impl<P, S, T> TransmutedSubplace<P, S, T>
where
    P: Subplace,
{
    pub unsafe fn new_unchecked(p: P) -> Self {
        Self(p, PhantomData, PhantomData)
    }
}
