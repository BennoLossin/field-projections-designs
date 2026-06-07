use std::mem::offset_of;

use field_projection_design::{
    application::refs::MutHandle,
    design::{
        Metadata,
        locals::LocalHandle,
        ops::{BorrowPlace, DerefPlace, ProjectPlace},
        subplace::Subplace,
    },
};

struct Struct {
    a: u32,
    b: i32,
}

struct StructA;
struct StructB;

unsafe impl Subplace for StructA {
    type Source = Struct;
    type Target = u32;

    fn offset(self, (): Metadata<Self::Source>) -> (usize, Metadata<Self::Target>) {
        (offset_of!(Struct, a), ())
    }
}

unsafe impl Subplace for StructB {
    type Source = Struct;
    type Target = i32;

    fn offset(self, (): Metadata<Self::Source>) -> (usize, Metadata<Self::Target>) {
        (offset_of!(Struct, b), ())
    }
}

/// ```
/// let mut x = Struct { a: 42, b: 24 };
/// let mut y = &mut x;
/// let z = &mut y;
///
/// let a = &mut (**z).a;
/// let b = &mut (**z).b;
/// *a += 1;
/// *b += 1;
/// ```
fn main() {
    let mut x = Struct { a: 42, b: 24 };
    let mut y = &mut x;
    let z = &mut y;

    let a: &mut u32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, u32> = ProjectPlace::<StructA>::project_place(hdl, StructA);
        BorrowPlace::<&mut u32>::borrow(hdl)
    };
    let b: &mut i32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, i32> = ProjectPlace::<StructB>::project_place(hdl, StructB);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };

    *a += 1;
    *b += 1;
}
