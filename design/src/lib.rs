#![expect(incomplete_features)]
// Needed for proper support of `?Sized` types.
#![feature(ptr_metadata)]
// Needed for enum support (`&'static str` in const generics).
#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![feature(box_vec_non_null)]
#![cfg_attr(doc, feature(custom_inner_attributes))]
#![feature(proc_macro_hygiene)]

use std::ptr::Pointee;

pub mod cell;
pub mod lang_limits;
pub mod mem;
pub mod ops;
pub mod option;
pub mod pin;
pub mod ptr;
pub mod sync;
pub mod vec;

pub type Metadata<T> = <T as Pointee>::Metadata;
