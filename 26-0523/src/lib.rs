#![feature(ptr_metadata)]

use std::ptr::Pointee;

pub mod borrowck;
pub mod locals;
pub mod ops;
pub mod ptrs;
pub mod refs;
pub mod subplace;

pub type Metadata<T> = <T as Pointee>::Metadata;
