use std::{
    marker::PhantomInvariant,
    ptr::Pointee,
};

use crate::{
    place::Subplace,
    ptr::Metadata,
};

pub struct TransmutedSubplace<Sub, Source: ?Sized, Target: ?Sized> {
    subplace: Sub,
    _source: PhantomInvariant<Source>,
    _target: PhantomInvariant<Target>,
}

impl<Sub: Default, Source: ?Sized, Target: ?Sized> Default
    for TransmutedSubplace<Sub, Source, Target>
{
    fn default() -> Self {
        Self {
            subplace: Default::default(),
            _source: Default::default(),
            _target: Default::default(),
        }
    }
}

unsafe impl<Sub, Source, Target> Subplace
    for TransmutedSubplace<Sub, Source, Target>
where
    Sub: Subplace,
    Source: ?Sized + Pointee<Metadata = Metadata<Sub::Source>>,
    Target: ?Sized + Pointee<Metadata = Metadata<Sub::Target>>,
{
    type Source = Source;

    type Target = Target;

    fn offset(
        self,
        metadata: Metadata<Self::Source>,
    ) -> (usize, Metadata<Self::Target>) {
        self.subplace.offset(metadata)
    }
}

impl<Sub, Source, Target> TransmutedSubplace<Sub, Source, Target>
where
    Sub: Subplace,
    Source: ?Sized,
    Target: ?Sized,
{
    pub unsafe fn new_unchecked(p: Sub) -> Self {
        Self {
            subplace: p,
            _source: PhantomInvariant::new(),
            _target: PhantomInvariant::new(),
        }
    }
}
