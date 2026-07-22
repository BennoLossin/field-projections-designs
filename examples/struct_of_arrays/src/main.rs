#![feature(phantom_variance_markers)]

use std::fmt::Display;

use design::{
    lang_limits::adt_reflect,
    ops::place::{
        BorrowPlace,
        DerefPlace,
        IndexPlace,
        ProjectPlace,
        ReadPlace,
        WritePlace,
    },
    place::LocalHandle,
    utils::example_helpers::struct_of_arrays::SoA,
};

mod soa;

use self::soa::{
    SoA,
    SoAHandle,
    SoAMut,
    SoARef,
};

adt_reflect!(
    /// 2d integer point.
    ///
    /// This type derives the `SoA` trait, which results in the following code
    /// being generated (some paths to types have been shortened for
    /// readability):
    ///
    /// ```
    #[doc = include_str!("derive_expansion.rs")]
    /// ```
    #[derive(SoA)]
    pub struct Point {
        x: i32,
        y: i32,
    }
);

#[expect(dead_code)]
#[doc(hidden)]
mod expansion {
    use super::*;
    #[derive(SoA)]
    struct Point {
        x: i32,
        y: i32,
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.x, self.y)
    }
}

fn print(value: impl Display) {
    println!("{value}");
}

/// ## Sugared Code
///
/// ```
/// let mut soa = SoAPoint { x: [0; 64], y: [0; 64] };
///
/// // Indexing into a SoA and writing/reading fields works:
/// soa[42].x = 42;
/// soa[42].y = -42;
/// print(soa[24].x);
///
/// // We can borrow a single element using a special `SoARef`:
/// let elem: SoARef<'_, Point, 64> = @soa[20];
/// // on that element, we can access the fields to read & borrow them
/// print(&elem.x);
/// print(&elem.y);
///
/// // The same can be done to mutate one element with `SoAMut`:
/// let elem: SoAMut<'_, Point, 64> = @soa[20];
/// elem.x = -1;
/// elem.y = -1;
///
/// // Pattern matching yields a slice of `SoARef` or `SoAMut`, since here we
/// // can determine disjointness:
/// match soa {
///     [] => print("no elements"),
///     [single] => print(&single.x),
///     [first, .., last] => {
///         first.x = last.y;
///         last.x = first.y;
///         print("more than one element")
///     }
/// }
/// ```
///
/// ## Desugared Code
///
/// ```
/// ==== DESUGARED ====
/// ```
#[design::utils::desugared]
fn main() {
    let mut soa = SoAPoint { x: [0; 64], y: [0; 64] };

    // Indexing into a SoA and writing/reading fields works:
    unsafe {
        let hdl: LocalHandle<SoAPoint<64>> = LocalHandle::new(&raw mut soa);
        let hdl: SoAHandle<_, LocalHandle<SoAPoint<64>>, _> =
            IndexPlace::index(hdl, 42);
        let subplace = <field_of!(Point, x)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        WritePlace::write_place(hdl, 42);
    }
    unsafe {
        let hdl: LocalHandle<SoAPoint<64>> = LocalHandle::new(&raw mut soa);
        let hdl: SoAHandle<_, LocalHandle<SoAPoint<64>>, _> =
            IndexPlace::index(hdl, 42);
        let subplace = <field_of!(Point, y)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        WritePlace::write_place(hdl, -42);
    }
    print(unsafe {
        let hdl: LocalHandle<SoAPoint<64>> = LocalHandle::new(&raw mut soa);
        let hdl: SoAHandle<_, LocalHandle<SoAPoint<64>>, _> =
            IndexPlace::index(hdl, 24);
        ReadPlace::read_place(hdl)
    });
    // We can borrow a single element using a special `SoARef`:
    let elem: SoARef<'_, Point, 64> = unsafe {
        let hdl: LocalHandle<SoAPoint<64>> = LocalHandle::new(&raw mut soa);
        let hdl: SoAHandle<_, LocalHandle<SoAPoint<64>>, _> =
            IndexPlace::index(hdl, 20);
        BorrowPlace::borrow(hdl)
    };
    // on that element, we can access the fields to read & borrow them
    print(unsafe {
        let hdl: LocalHandle<SoARef<'_, Point, 64>> =
            LocalHandle::new(&raw const elem);
        let hdl: SoAHandle<Point, _, 64> = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(Point, x)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        BorrowPlace::<&_>::borrow(hdl)
    });
    print(unsafe {
        let hdl: LocalHandle<SoARef<'_, Point, 64>> =
            LocalHandle::new(&raw const elem);
        let hdl: SoAHandle<Point, _, 64> = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(Point, y)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        BorrowPlace::<&_>::borrow(hdl)
    });
    // The same can be done to mutate one element with `SoAMut`:
    let elem: SoAMut<'_, Point, 64> = unsafe {
        let hdl: LocalHandle<SoAPoint<64>> = LocalHandle::new(&raw mut soa);
        let hdl: SoAHandle<_, LocalHandle<SoAPoint<64>>, _> =
            IndexPlace::index(hdl, 20);
        BorrowPlace::borrow(hdl)
    };
    unsafe {
        let hdl: LocalHandle<SoAMut<'_, Point, 64>> =
            LocalHandle::new(&raw const elem);
        let hdl: SoAHandle<Point, _, 64> = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(Point, x)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        WritePlace::write_place(hdl, -1);
    }
    unsafe {
        let hdl: LocalHandle<SoAMut<'_, Point, 64>> =
            LocalHandle::new(&raw const elem);
        let hdl: SoAHandle<Point, _, 64> = DerefPlace::deref_place(hdl);
        let subplace = <field_of!(Point, y)>::default();
        let hdl = ProjectPlace::project_place(hdl, subplace);
        WritePlace::write_place(hdl, -1);
    }
    // Pattern matching yields a slice of `SoARef` or `SoAMut`, since here we
    // can determine disjointness:
    /* TODO:
        match soa {
            [] => print("no elements"),
            [single] => print("only one element"),
            [first, .., last] => {
                first.x = last.y;
                last.x = first.y;
            }
        }
    */
}
