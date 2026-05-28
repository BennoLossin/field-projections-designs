use std::{marker::PhantomData, ptr::Pointee};

use crate::Metadata;

pub unsafe trait Subplace: Sized {
    type Source: ?Sized;
    type Target: ?Sized;

    fn offset(self, metadata: Metadata<Self::Source>) -> (usize, Metadata<Self::Target>);
}

pub struct TransmutedSubplace<P, S: ?Sized, T: ?Sized>(P, PhantomData<S>, PhantomData<T>);

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

impl<P, S, T> TransmutedSubplace<P, S, T>
where
    P: Subplace,
{
    pub unsafe fn new_unchecked(p: P) -> Self {
        Self(p, PhantomData, PhantomData)
    }
}
