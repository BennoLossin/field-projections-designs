use std::marker::PhantomData;

pub trait Timing {}

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
