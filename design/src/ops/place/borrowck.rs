use std::marker::PhantomData;

use macros::sealed;

#[sealed]
pub trait Timing {}

pub struct Instant;
pub struct Lifetime<'a>(PhantomData<&'a ()>);
pub struct UntilDrop;

#[sealed]
impl Timing for Instant {}
#[sealed]
impl Timing for Lifetime<'_> {}
#[sealed]
impl Timing for UntilDrop {}

pub enum AccessKind {
    Shared,
    Exclusive,
    Untracked,
}

#[sealed]
pub trait Access {
    const KIND: AccessKind;
}

pub struct Shared;
pub struct Exclusive;
pub struct Untracked;

#[sealed]
impl Access for Shared {
    const KIND: AccessKind = AccessKind::Shared;
}
#[sealed]
impl Access for Exclusive {
    const KIND: AccessKind = AccessKind::Exclusive;
}
#[sealed]
impl Access for Untracked {
    const KIND: AccessKind = AccessKind::Untracked;
}

#[sealed]
pub trait AtLeastShared: Access {}

#[sealed]
impl AtLeastShared for Shared {}
#[sealed]
impl AtLeastShared for Exclusive {}
