use std::{marker::PhantomData, ptr::Pointee};

use crate::design::Metadata;

pub unsafe trait Subplace: Sized {
    type Source: ?Sized;
    type Target: ?Sized;

    fn offset(self, metadata: Metadata<Self::Source>) -> (usize, Metadata<Self::Target>);
}

pub struct TransmutedSubplace<Sub, Source: ?Sized, Target: ?Sized>(
    Sub,
    PhantomData<Source>,
    PhantomData<Target>,
);

unsafe impl<Sub, Source, Target> Subplace for TransmutedSubplace<Sub, Source, Target>
where
    Sub: Subplace,
    Source: ?Sized + Pointee<Metadata = Metadata<Sub::Source>>,
    Target: ?Sized + Pointee<Metadata = Metadata<Sub::Target>>,
{
    type Source = Source;

    type Target = Target;

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
