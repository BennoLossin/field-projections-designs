use std::marker::PhantomData;

pub trait Timing: sealed::Sealed {}

pub struct Instant;
pub struct Lifetime<'a>(PhantomData<&'a ()>);
pub struct UntilDrop;

impl Timing for Instant {}
impl Timing for Lifetime<'_> {}
impl Timing for UntilDrop {}

pub enum AccessKind {
    Shared,
    Exclusive,
    Untracked,
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Instant {}
    impl Sealed for super::Lifetime<'_> {}
    impl Sealed for super::UntilDrop {}
}
