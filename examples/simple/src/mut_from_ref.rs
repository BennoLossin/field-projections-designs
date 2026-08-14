/// FIXME: This should not compile!
///
/// ## Sugared Code
///
/// ```
/// let mut x: i32 = 42;
/// let y = &&mut x;
/// let a = &mut (**y); // yikes!
/// *a += 1;
/// dbg!(x);
/// ```
///
/// ## Desugared code
///
/// ```
/// ==== DESUGARED ====
/// ```
#[cfg_attr(doc, design::utils::desugared)]
pub fn problematic() {
    let mut x = 42;
    let mut y1 = unsafe {
        let hdl: LocalHandle<i32> = LocalHandle::new(&raw mut x);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };
    let mut y = unsafe {
        let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw mut y1);
        BorrowPlace::<&&mut i32>::borrow(hdl)
    };
    let a: &mut i32 = unsafe {
        let hdl: LocalHandle<&&mut i32> = LocalHandle::new(&raw mut y);
        let hdl: RefHandle<&mut i32> = DerefPlace::deref_place(hdl);
        let hdl: MutHandle<i32> = DerefPlace::deref_place(hdl);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };
    *a += 1;
    dbg!(x);
}
