#![expect(incomplete_features)]
// Needed for proper support of `?Sized` types.
#![feature(ptr_metadata)]
// Needed for enum support (`&'static str` in const generics).
#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![cfg_attr(doc, feature(custom_inner_attributes))]
#![feature(proc_macro_hygiene)]

use std::ptr::Pointee;

pub mod application;
pub mod borrowck;
pub mod compat;
pub mod enums;
pub mod fallible;
pub mod locals;
pub mod ops;
pub mod subplace;

pub type Metadata<T> = <T as Pointee>::Metadata;

/// Generate reflection information for ADTs.
///
/// The input should be structs and enums, which will be emitted as-is and in addition, reflection
/// information will be generated. This information is exposed by generating two macros:
/// - `field_of!($ty:ty, $field:ident)`
///
///   this macro returns the field representing type of the given field. It implements the
///   [`Subplace`] trait.
///
///   Note that this macro also supports fields of enum variants via the syntax
///   `field_of!(MyEnum::Variant, field)`.
/// - `variant_of!($enum:ty, $variant:ident)`
///
///   this macro returns the variant type of the given variant, which has the same layout as the
///   enum itself, but is statically guaranteed to be in the given variant. In pattern type terms,
///   this is just `pattern_type!($enum is $variant { .. })` or `pattern_type!($enum is
///   $variant(..))` depending on the variant having named or unnamed fields.
///
/// [`Subplace`]: crate::subplace::Subplace
pub use macros::adt_reflect;
