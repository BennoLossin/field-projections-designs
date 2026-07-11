//! Basic place operation examples.
//!
//! This example crate contains some simple examples along with explanations to
//! help the reader get acquainted with how place operations are desugared.
//! The examples contain both the sugared and desugared versions of the code.
//! The desugared ones also contain all type ascriptions to make reading the
//! code easier.
//!
//! The first example function that you should take a look at is the [`basic`]
//! function, as it explains the desugaring that happens. It also is the
//! simplest of the examples that is available.
//!
//! After that, [`nested_mut`] showcases an interesting and -- while intuitive
//! -- surprising property of [`&mut T`](primitive@reference).
//!
//! Lastly, we showcase how pinning a mutable reference looks like in [`pin`].

use std::{
    fmt::Display,
    ops::AddAssign,
};

use design::{
    lang_limits::adt_reflect,
    ops::place::{
        BorrowPlace,
        DerefPlace,
        DropPlace,
        ProjectPlace,
        ReadPlace,
        WritePlace,
    },
    place::{
        LocalHandle,
        MutHandle,
    },
};

adt_reflect!(
    pub struct Struct {
        a: u32,
        b: i32,
    }
);

fn print<T: Display>(value: T) {
    println!("{value}");
}

include!("basic.rs");
include!("nested_mut.rs");

pub fn main() {
    basic();
    nested_mut();
}
