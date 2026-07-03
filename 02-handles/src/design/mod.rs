use std::ptr::Pointee;

pub mod borrowck;
pub mod enums;
pub mod fallible;
pub mod locals;
pub mod ops;
pub mod subplace;

pub type Metadata<T> = <T as Pointee>::Metadata;
