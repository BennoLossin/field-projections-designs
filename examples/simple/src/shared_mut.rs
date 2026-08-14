#[test]
#[doc(hidden)]
fn shared_mut_sugared() {
    let mut x: i32 = 42;
    let y1 = &mut x;
    let y = &y1;
    let a = &**y; // would error with `&mut **y`
    dbg!(a);
}

/// Nesting a mutable reference behind a shared reference disables mutability.
///
/// ## Sugared Code
///
/// ```
/// let mut x: i32 = 42;
/// let y1 = &mut x;
/// let y = &y1;
/// let a = &**y; // would error with `&mut **y`
/// dbg!(a);
/// ```
///
/// ## Desugared Code
///
/// ```
/// ==== DESUGARED ====
/// ```
///
/// If the value bound in `let a` was `&mut **y` instead, the desugaring would
/// no longer compile:
///
/// ```
/// let a: &i32 = unsafe {
///     let hdl: LocalHandle<&&mut i32> = LocalHandle::new(&raw mut y);
///     let hdl: RefHandle<&mut i32> = DerefPlace::deref_place(hdl);
///     let hdl: RefHandle<i32> = DerefPlace::deref_place(hdl);
///     BorrowPlace::<&mut i32>::borrow(hdl)
/// //  ------------------------------- ^^^
/// //  the trait `BorrowPlace<&mut i32>` is not implemented for `RefHandle<'_, i32>`
/// //  required by a bound introduced by this call
/// };
/// ```
#[cfg_attr(test, test)]
#[cfg_attr(doc, design::utils::desugared)]
pub fn shared_mut() {
    let mut x = 42;
    let mut y1 = unsafe {
        let hdl: LocalHandle<i32> = LocalHandle::new(&raw mut x);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };
    let mut y = unsafe {
        let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw mut y1);
        BorrowPlace::<&&mut i32>::borrow(hdl)
    };
    let a: &i32 = unsafe {
        let hdl: LocalHandle<&&mut i32> = LocalHandle::new(&raw mut y);
        let hdl: RefHandle<&mut i32> = DerefPlace::deref_place(hdl);
        let hdl: RefHandle<i32> = DerefPlace::deref_place(hdl);
        BorrowPlace::<&i32>::borrow(hdl)
    };
    dbg!(a);
}
