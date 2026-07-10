use design::{
    lang_limits::adt_reflect,
    ops::place::{
        BorrowPlace,
        DerefPlace,
        ProjectPlace,
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

/// A small example showing the power of the place based approach over
/// [`DerefMut`].
///
/// ## Sugared Code
///
/// ```ignore
/// let mut x = Struct { a: 42, b: 24 };
/// let mut y = &mut x;
/// let z = &mut y;
///
/// let a = &mut (**z).a;
/// let b = &mut z.b;
/// *a += 1;
/// *b += 1;
/// ```
/// There are several things happening here that are notable:
/// - in the assignment of `b`, there are implicit derefs of `z`.
/// - we're able to use both `a` *and* `b`; if `&mut` was implemented with
///   [`DerefMut`] instead, there would be two calls to `deref_mut`. The second
///   one (the one that creates `b`) would invalidate `a`.
/// - the mutable reference that we're accessing sits behind another mutable
///   reference, this makes it more complex for the borrow checker tracking, as
///   one could overwrite the inner mutable reference without invalidating
///   borrows of it.
///
///
/// ## Desugared code
///
/// The desugaring is rather straight forward, but can be very intimidating for
/// the first time. Here's a quick rundown what the desugaring process looks like:
///
/// 1. Place expressions are made explicit. In this case, the subexpression
///    `z.b` in the `let b` statement is expanded to `(**z).b`.
/// 2. Temporary variables are made explicit (in this example, there are no
///    temporaries).
/// 3. Place expressions are recursively turned into place handles:
///    - Local variable accesses of `$var` are turned into <code>[LocalHandle]::new(&raw const $var)</code>
///    - Dereference operations of `$hdl` are turned into
///      <code>[DerefPlace]::deref_place($hdl)</code>
///    - Field access operations of the handle `$hdl` with the field `$field`
///      are turned into <code>[ProjectPlace]::project_place($hdl, $subplace)</code>
///      where `$subplace` is the [`Subplace`]-representing value created based
///      on the type that `$hdl` points at and the field `$field`.
/// 4. Place operations on place expressions are replaced by the respective
///    operation on the computed handle:
///    - [`BorrowPlace`]
///    - [`ReadPlace`](design::ops::place::ReadPlace) (not shown here)
///    - [`WritePlace`](design::ops::place::WritePlace) (not shown here)
///
/// Note that the borrow checker is intended to run before this desugaring,
/// which is why the desugared `unsafe` is going to be sound.
///
/// ```ignore
/// ==== DESUGARED ====
/// ```
///
/// [`DerefMut`]: core::ops::DerefMut
/// [`Subplace`]: design::place::Subplace
#[cfg_attr(test, test)]
#[cfg_attr(doc, design::utils::desugared)]
pub fn nested_mut() {
    let x = Struct { a: 42, b: 24 };
    let y = unsafe {
        let hdl: LocalHandle<Struct> = LocalHandle::new(&raw const x);
        BorrowPlace::<&mut Struct>::borrow(hdl)
    };
    let z = unsafe {
        let hdl: LocalHandle<&mut Struct> = LocalHandle::new(&raw const y);
        BorrowPlace::<&mut &mut Struct>::borrow(hdl)
    };

    let a: &mut u32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let a_subplace = <field_of!(Struct, a)>::default();
        let hdl: MutHandle<'_, u32> =
            ProjectPlace::project_place(hdl, a_subplace);
        BorrowPlace::<&mut u32>::borrow(hdl)
    };
    let b: &mut i32 = unsafe {
        let hdl: LocalHandle<&mut &mut Struct> = LocalHandle::new(&raw const z);
        let hdl: MutHandle<'_, &mut Struct> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<'_, Struct> = DerefPlace::deref_place(hdl);
        let b_subplace = <field_of!(Struct, b)>::default();
        let hdl: MutHandle<'_, i32> =
            ProjectPlace::project_place(hdl, b_subplace);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };

    *a += 1;
    *b += 1;
}

pub fn main() {
    nested_mut();
}
