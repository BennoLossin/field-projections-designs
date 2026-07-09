#![expect(incomplete_features)]
// Needed for proper support of `?Sized` types.
#![feature(ptr_metadata)]
// Needed for enum support (`&'static str` in const generics).
#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![cfg_attr(doc, feature(custom_inner_attributes))]
#![feature(proc_macro_hygiene)]

pub mod application;
pub mod design;

pub use adt_reflect::adt_reflect;
