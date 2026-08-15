use crate::{
    ops::place::{
        BorrowPlace,
        IndexPlace,
        Indexable,
        PlaceHandle,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
        },
    },
    place::{
        LocalHandle,
        RefHandle,
    },
};

impl<T, const N: usize> Indexable<usize> for [T; N] {
    type Element = T;
}

// TODO: Make this generic over the handle type
//
// The problem is that we have no generic way to say "create an equivalent
// handle of another type after doing pointer arithmetic". Our options are
//
// 1. Wrap the handle with our own type and do impls of all the place ops
//    conditional on the wrapped handle type
//      * This is clunky and annoying, but seems possible. Supertrait auto impl
//        might make it more pleasant with a `HandleProxy` convenience trait.
// 2. Do impls for `H: BorrowPlace<&T>` and `H: BorrowPlace<&mut T>`, which is
//    the moral equivalent of today's Index/IndexMut impls.
//      * This bakes in the reference kind. *Maybe* this is correct, because
//        we can't say what indexing other kinds of references should allow,
//        but then it should be a trait on the reference type so you can impl
//        it for your own.
unsafe impl<'a, T, const N: usize>
    IndexPlace<usize, LocalHandle<Self>, Lifetime<'a>, Instant> for [T; N]
{
    type ElementHandle = LocalHandle<T>;

    const POINTEE_ACCESS: AccessKind = AccessKind::Exclusive;
    const POINTER_ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    fn index(handle: LocalHandle<Self>, idx: usize) -> Self::ElementHandle {
        if idx >= N {
            panic!()
        }
        // SAFETY: Bounds check above.
        unsafe { LocalHandle::new(handle.as_ptr().cast::<T>().add(idx)) }
    }
}

impl<T> Indexable<usize> for [T] {
    type Element = T;
}

// Example of option #2 from above. Notice that either
//
// 1. There is a single, handle-agnostic IndexPlace impl for [T] that hard codes
//    &[T] as the output place, or
// 2. There needs to be a copy of this impl for every handle type.
//
// To see why hard coding references is necessary in (1), notice that we must
// provide some input parameter to `BorrowPlace`, and there is no "canonical"
// reference type we can get from the handle itself. Perhaps if we had canonical
// types to use with `@expr`, we could use those?
//
// Otherwise, (2) sounds preferable since we can stuff the impl in a derive.
unsafe impl<'a, T: 'a, H> IndexPlace<usize, H, Lifetime<'a>, Instant> for [T]
where
    H: PlaceHandle<Target = [T]> + BorrowPlace<&'a [T]>,
{
    type ElementHandle = RefHandle<'a, T>;

    const POINTEE_ACCESS: AccessKind = AccessKind::Shared;
    const POINTER_ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    fn index(handle: H, idx: usize) -> Self::ElementHandle {
        let elem: *const T = &raw const unsafe { handle.borrow() }[idx];
        unsafe { RefHandle::from_raw(elem) }
    }
}

#[cfg(false)]
unsafe impl<'b, H, T: 'b> IndexPlace<usize, H, Lifetime<'b>, Instant> for [T]
where
    H: PlaceHandle<Target = [T]> + BorrowPlace<&'b mut [T]>,
{
    type ElementHandle = MutHandle<'b, T>;
    const POINTEE_ACCESS: AccessKind = AccessKind::Exclusive;
    const POINTER_ACCESS: AccessKind = AccessKind::Exclusive;
    const SAFE: bool = true;

    fn index(handle: H, idx: usize) -> Self::ElementHandle {
        let slice: &'b mut [T] = unsafe { H::borrow(handle) };
        let elem_ptr: *mut T = &mut slice[idx];
        unsafe { MutHandle::from_raw(elem_ptr) }
    }
}
