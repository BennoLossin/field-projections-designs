//! Operations on places.
//!
//! <details><summary>Birds-eye view of this module</summary>
//!
//! ```ignore
#![doc = macros::raw_summary!()]
//! ```
//!
//! </details>
//!
//! ## Overview
//!
//! This module contains traits to customize the place operations of Rust:
//! - reading,
//! - writing, and
//! - borrowing.
//!
//! ## Places
//!
//! A *place* in Rust is a particular location in memory. Places are represented
//! by [*place expressions*][ref-place-exprs], which have the following form:
//! - `$path`: paths that refer to locals variables (also parameters) and
//!   statics,
//! - `*$place`: dereferences of another place expression,
//! - `$place[$expr]`: indexing into another place expression,
//! - `$place.$ident`: accessing a field of another place expression,
//! - `($place)`: parenthesized place expressions,
//! - `$value`: value expressions can be coerced to a temporary place whose
//!   lifetime is determined from its context,
//!
//! Further reading:
//! - <https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html>
//! - <https://www.ralfj.de/blog/2024/08/14/places.html>
//!
//! [ref-place-exprs]: https://doc.rust-lang.org/reference/expressions.html#r-expr.place-value.place-expr-kinds
//!
//! Place expressions have a direct representation in the form of a
//! [`PlaceHandle`]. A handle points at a place and is responsible for
//! performing all available place operations on the represented place. Any
//! valid place expression is converted into a handle by the compiler:
//!
//! - a [`LocalHandle`] is created whenever a static, local variable, or
//!   temporary is accessed,
//! - [`DerefHandle`] allows dereferencing a handle, producing
//!   another handle that now points at the pointee of the original place,
//! - [`ProjectPlace`] allows to access a subplace of a place, yielding a
//!   handle to that subplace,
//! - [`IndexPlace`] ...?
//!
//! ### Place Wrappers
//!
//! Place wrappers are a special kind of place proxy. They "physically contain"
//! the place they are proxying for. A good example is [`MaybeUninit<T>`]. To
//! support subplaces of these place wrappers, the [`WrapPlace`] trait exists.
//! It allows forwarding subplaces to the proxy and changing the subplace access
//! information. With [`MaybeUninit<T>`], this allows accessing any subplace
//! under the transformation that it's type is wrapped in `MaybeUninit`. So
//! Given `&MaybeUninit<Struct>`, the `field` subplace can be borrowed using `&`
//! and it has type `&MaybeUninit<Field>`.
//!
//! [`MaybeUninit<T>`]: std::mem::MaybeUninit
//!
//! ### Safety
//!
//! All operation functions are `unsafe`, since they have raw pointer arguments
//! that have safety preconditions. The arguments are raw pointers, because the
//! values they point to need not be in a valid state (they may be partially
//! moved out or borrowed).
//!
//! The safety requirements for the operation functions have not been figured
//! out at this point in time. Since we expect several changes to the design, we
//! do not want to commit to writing down good safety documentation before
//! having finished the design.
//!
//! What is clear at the moment is that the safety requirements will heavily
//! interact with the borrow checker. It will ensure that simultaneous place
//! operations on the same value are allowed, since they either affect disjoint
//! subplaces, or because they both only require shared access. For example:
//! - reading `ptr.field.subfield` and borrowing `ptr.field` with `&T` are
//!   allowed to happen at the same time,
//! - writing `ptr.field` and borrowing `ptr.field.subfield` at the same time is
//!   not allowed.
//!
//! The safety of using the place operations via the operators will depend on
//! the value of the `SAFE` constant in the operation traits. At the moment we
//! will only permit a literal value of `true` or `false` in implementations. It
//! will dictate if people have to write for example `unsafe { &*ptr }` or if
//! `*ptr` is allowed. It should be set to `true` when the borrow checker's
//! guarantees of either disjoint subplaces or "all concurrent operations are
//! shared" are enough to calling the operations' function correctly. If there
//! are additional requirements, such as "ptr is valid", then `SAFE` should be
//! set to `false`. For example, `&mut T` will have `SAFE = true` in
//! [`ReadPlace`], but `NonNull<T>` will set it to `false`.

#[macros::summary(skip)]
use crate::{
    Metadata,
    ops::place::{
        borrowck::{
            AccessKind,
            Timing,
        },
        subplace::{
            HasVariant,
            Matchable,
            Subplace,
            VariantType,
        },
    },
};

#[macros::summary(skip)]
pub mod borrowck;
#[macros::summary(skip)]
pub mod fallible;
#[macros::summary(skip)]
pub mod subplace;

#[macros::summary(skip)]
mod locals;
#[macros::summary(skip)]
mod reference;

#[macros::summary(skip)]
pub use self::{
    locals::LocalHandle,
    reference::{
        MutHandle,
        RefHandle,
    },
};

pub trait ProxyPlace {
    type Handle: PlaceHandle;
}

/// A *handle* to a place.
pub trait PlaceHandle: Sized {
    type Target: ?Sized;
}

pub trait DerefHandle: ProxyPlace {
    const ACCESS: AccessKind;
    type Timing: Timing;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle;
}

pub trait ReadPlace: PlaceHandle {
    const ACCESS: AccessKind;
    const SAFE: bool;

    unsafe fn read_place(self) -> Self::Target;
}

pub trait ReadMetadata: PlaceHandle {
    fn metadata(self) -> Metadata<Self::Target>;
}

pub trait ReadVariant: PlaceHandle
where
    Self::Target: Matchable,
{
    unsafe fn read_variant(self) -> &'static str;
}

pub trait VariantPlace<const VARIANT: &'static str>: ReadVariant
where
    Self::Target: Matchable,
    Self::Target: HasVariant<VARIANT>,
{
    type ToVariant: PlaceHandle<Target = VariantType<Self::Target, VARIANT>>;

    unsafe fn cast(self) -> Self::ToVariant;
}

pub trait MovePlace: ReadPlace {
    const ACCESS: AccessKind;
    const SAFE: bool;
}

pub trait WritePlace: PlaceHandle {
    const ACCESS: AccessKind;
    const SAFE: bool;

    unsafe fn write_place(self, value: Self::Target);
}

pub trait DropPlace: PlaceHandle {
    unsafe fn drop_place(self);
}

pub trait DropHusk: ProxyPlace {
    unsafe fn drop_husk(this: Self::Handle);
}

pub trait ProjectPlace<S>: PlaceHandle
where
    S: Subplace<Source = Self::Target>,
{
    type Projected: PlaceHandle<Target = S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected;
}

pub trait DerefPlace<PointeeTiming, PointerTiming>: PlaceHandle
where
    Self::Target: ProxyPlace,
    PointeeTiming: Timing,
    PointerTiming: Timing,
{
    const POINTEE_ACCESS: AccessKind;
    const POINTER_ACCESS: AccessKind;
    const SAFE: bool;

    unsafe fn deref_place(self) -> <Self::Target as ProxyPlace>::Handle;
}

pub trait Indexable<Idx> {
    type Element: ?Sized;
}

/// `place[idx]`
pub trait IndexPlace<Idx, H>: Indexable<Idx>
where
    H: PlaceHandle<Target = Self>,
{
    type ElementHandle: PlaceHandle<Target = Self::Element>;

    fn index(self, idx: Idx) -> Self::ElementHandle;
}

pub trait BorrowPlace<Output>: PlaceHandle {
    const ACCESS: AccessKind;
    type Timing: Timing;
    const SAFE: bool;

    unsafe fn borrow(self) -> Output;
}

pub trait PlaceWrapper {
    type Inner: ?Sized;
}

pub unsafe trait WrapPlace<S>: PlaceWrapper
where
    S: Subplace<Source = Self::Inner>,
{
    type Wrapped: Subplace<Source = Self>;

    fn wrap(subplace: S) -> Self::Wrapped;
}
