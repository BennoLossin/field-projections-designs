use _02_handles::{
    application::refs::MutHandle,
    design::{
        locals::LocalHandle,
        ops::{BorrowPlace, DerefPlace, ProjectPlace},
    },
};
use adt_reflect::adt_reflect;

adt_reflect!(
    pub struct Struct {
        a: u32,
        b: i32,
    }
);

/// ```
/// let mut x = Struct { a: 42, b: 24 };
/// let mut y = &mut x;
/// let z = &mut y;
///
/// let a = &mut (**z).a;
/// let b = &mut z.b;
/// *a += 1;
/// *b += 1;
/// ```
#[cfg_attr(test, test)]
fn main() {
    let mut x = Struct { a: 42, b: 24 };
    let mut y = &mut x;
    let z = &mut y;

    let a: &mut u32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let a_subplace = <field_of!(Struct, a)>::default();
        let hdl: MutHandle<'_, u32> = ProjectPlace::project_place(hdl, a_subplace);
        BorrowPlace::<&mut u32>::borrow(hdl)
    };
    let b: &mut i32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let b_subplace = <field_of!(Struct, b)>::default();
        let hdl: MutHandle<'_, i32> = ProjectPlace::project_place(hdl, b_subplace);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };

    *a += 1;
    *b += 1;
}
