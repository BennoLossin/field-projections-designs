use std::{
    marker::PhantomData,
    ptr::Pointee,
};

use crate::ptr::Metadata;

pub unsafe trait Subplace: Sized {
    type Source: ?Sized;
    type Target: ?Sized;

    fn offset(
        self,
        metadata: Metadata<Self::Source>,
    ) -> (usize, Metadata<Self::Target>);
}

pub struct TransmutedSubplace<Sub, Source: ?Sized, Target: ?Sized>(
    Sub,
    PhantomData<Source>,
    PhantomData<Target>,
);

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
        self.0.offset(metadata)
    }
}

impl<Sub, Source, Target> TransmutedSubplace<Sub, Source, Target>
where
    Sub: Subplace,
    Source: ?Sized,
    Target: ?Sized,
{
    pub unsafe fn new_unchecked(p: Sub) -> Self {
        Self(p, PhantomData, PhantomData)
    }
}

pub unsafe trait Matchable {
    const VARIANTS: &'static [&'static str];

    unsafe fn variant_at(ptr: *const Self) -> &'static str;
}

pub unsafe trait HasVariant<const VARIANT: &'static str>:
    Matchable
{
    type VariantType;
}

pub type VariantType<E, const VARIANT: &'static str> =
    <E as HasVariant<VARIANT>>::VariantType;
