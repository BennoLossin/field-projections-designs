#[test]
#[doc(hidden)]
fn borrow_mut_array_index_sugared() {
    let mut array: [i32; 3] = [1, 2, 3];
    let x: &mut i32 = &mut array[1];
    *x = 4;
    assert_eq!(array, [1, 4, 3]);
}

/// ## Sugared code
///
/// ```
/// let mut array: [i32; 3] = [1, 2, 3];
/// let x: &mut i32 = &mut array[1];
/// *x = 4;
/// assert_eq!(array, [1, 4, 3]);
/// ```
///
/// ## Desugared code
///
/// ```
/// ==== DESUGARED ====
/// ```
#[cfg_attr(doc, design::utils::desugared)]
pub fn borrow_mut_array_index() {
    let mut array: [i32; 3] = [1, 2, 3];
    let x: &mut i32 = unsafe {
        let hdl: LocalHandle<[i32; 3]> = LocalHandle::new(&raw mut array);
        let hdl: LocalHandle<i32> = IndexPlace::index(hdl, 1);
        BorrowPlace::<&mut i32>::borrow(hdl)
    };
    unsafe {
        let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw const x);
        let hdl: MutHandle<'_, i32> = DerefPlace::deref_place(hdl);
        WritePlace::write_place(hdl, 4);
    }
    assert_eq!(array, [1, 4, 3]);
}

#[test]
#[doc(hidden)]
fn borrow_slice_index_sugared() {
    let array = [1, 2, 3];
    let slice: &[i32] = array.as_slice();
    let x: &i32 = &slice[1];
    match slice {
        [_, 2, _] => (),
        _ => panic!(),
    }
    assert_eq!(*x, 2);
}

/// ## Sugared code
///
/// ```
/// let array = [1, 2, 3];
/// let slice: &[i32] = &array;
/// let x: &i32 = &slice[1];
/// assert_eq!(*x, 2);
/// ```
///
/// ## Desugared code
///
/// ```
/// ==== DESUGARED ====
/// ```
#[cfg_attr(doc, design::utils::desugared)]
pub fn borrow_slice_index() {
    let array = [1, 2, 3];
    let slice: &[i32] = array.as_slice();
    let x: &i32 = unsafe {
        let hdl: LocalHandle<&[i32]> = LocalHandle::new(&raw const slice);
        // FIXME(tmandry): Does deref automatically happen at this level?
        // Should we be implementing on `&[T]`, or writing an IndexPlace impl
        // that's generic over DerefPlace?
        let hdl: RefHandle<[i32]> = DerefPlace::deref_place(hdl);
        let hdl: RefHandle<i32> = IndexPlace::index(hdl, 1);
        BorrowPlace::<&i32>::borrow(hdl)
    };
    assert_eq!(*x, 2);
}
