#![allow(incomplete_features)]
#![feature(generic_const_items)]
#![cfg_attr(not(test), expect(unused))]

use std::marker::PhantomData;

use design::{
    ops::place::{
        BorrowPlace,
        DerefPlace,
        IndexPlace,
        Indexable,
        PlaceHandle,
        ProjectPlace,
        WritePlace,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
        },
    },
    place::{
        LocalHandle,
        MutHandle,
        Subplace,
    },
};

pub struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq)]
pub struct SoAPoint<const N: usize> {
    xs: [i32; N],
    ys: [i32; N],
}

// TODO(tmandry): Manually expanded until the macro supports us.
#[doc(hidden)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
mod ___lang_limits {
    use super::*;
    pub struct ___Point__x(::core::marker::PhantomData<Point>);

    impl ::core::default::Default for ___Point__x {
        fn default() -> Self {
            Self(::core::marker::PhantomData)
        }
    }
    unsafe impl ::design::place::Subplace for ___Point__x {
        type Source = Point;
        type Target = i32;
        fn offset(self, (): ()) -> (::core::primitive::usize, ()) {
            (::core::mem::offset_of!(Point, x,), ())
        }
    }
    pub struct ___Point__y(::core::marker::PhantomData<Point>);

    impl ::core::default::Default for ___Point__y {
        fn default() -> Self {
            Self(::core::marker::PhantomData)
        }
    }
    unsafe impl ::design::place::Subplace for ___Point__y {
        type Source = Point;
        type Target = i32;
        fn offset(self, (): ()) -> (::core::primitive::usize, ()) {
            (::core::mem::offset_of!(Point, y,), ())
        }
    }
    pub struct ___SoAPoint__xs<const N: usize>(
        ::core::marker::PhantomData<SoAPoint<N>>,
    );

    impl<const N: usize> ::core::default::Default for ___SoAPoint__xs<N> {
        fn default() -> Self {
            Self(::core::marker::PhantomData)
        }
    }
    unsafe impl<const N: usize> ::design::place::Subplace for ___SoAPoint__xs<N> {
        type Source = SoAPoint<N>;
        type Target = [i32; N];
        fn offset(self, (): ()) -> (::core::primitive::usize, ()) {
            (::core::mem::offset_of!(SoAPoint<N>, xs,), ())
        }
    }
    pub struct ___SoAPoint__ys<const N: usize>(
        ::core::marker::PhantomData<SoAPoint<N>>,
    );

    impl<const N: usize> ::core::default::Default for ___SoAPoint__ys<N> {
        fn default() -> Self {
            Self(::core::marker::PhantomData)
        }
    }
    unsafe impl<const N: usize> ::design::place::Subplace for ___SoAPoint__ys<N> {
        type Source = SoAPoint<N>;
        type Target = [i32; N];
        fn offset(self, (): ()) -> (::core::primitive::usize, ()) {
            (::core::mem::offset_of!(SoAPoint<N>, ys,), ())
        }
    }

    pub type ___SoAPoint__xs__ctor<const N: usize> = ___SoAPoint__xs<N>;
    pub const ___SoAPoint__xs__ctor<const N: usize>: ___SoAPoint__xs<N> = ___SoAPoint__xs(PhantomData);
    pub type ___SoAPoint__ys__ctor<const N: usize> = ___SoAPoint__ys<N>;
    pub const ___SoAPoint__ys__ctor<const N: usize>: ___SoAPoint__ys<N> = ___SoAPoint__ys(PhantomData);
}

#[doc(hidden)]
macro_rules! field_of {
    (Point,x) => {
        ___lang_limits::___Point__x
    };
    (Point,y) => {
        ___lang_limits::___Point__y
    };
    (SoAPoint$(::)? < $N:path$(,)? > ,xs) => {
        ___lang_limits::___SoAPoint__xs__ctor::<N>
    };
    (SoAPoint$(::)? < $N:path$(,)? > ,ys) => {
        ___lang_limits::___SoAPoint__ys__ctor::<N>
    };
    ($($fallback:tt)*) => {
        ::core::compile_error!("unknown type, variant, or field")
    };
}

unsafe trait PointField<const N: usize>:
    Subplace<Source = Point, Target: Sized>
{
    type SoAField: Default + Subplace<Source = SoAPoint<N>>;
}

unsafe impl<const N: usize> PointField<N> for field_of!(Point, x) {
    type SoAField = field_of!(SoAPoint<N>, xs);
}

unsafe impl<const N: usize> PointField<N> for field_of!(Point, y) {
    type SoAField = field_of!(SoAPoint<N>, ys);
}

pub struct SoAElementPoint<'a, H> {
    idx: usize,
    inner: H,
    _lt: PhantomData<&'a SoAPoint<1>>,
}

impl<'a, H> PlaceHandle for SoAElementPoint<'a, H> {
    type Target = Point;
}

impl<const N: usize> Indexable<usize> for SoAPoint<N> {
    type Element = Point;
}

unsafe impl<'a, H, const N: usize> IndexPlace<usize, H, Lifetime<'a>, Instant>
    for SoAPoint<N>
where
    H: PlaceHandle<Target = Self>,
{
    type ElementHandle = SoAElementPoint<'a, H>;

    // FIXME(tmandry): This isn't right. How do we know??
    // Probably by replicating the whole H/J/K dance on the ProjectPlace impl below.
    const POINTEE_ACCESS: AccessKind = AccessKind::Exclusive;
    const POINTER_ACCESS: AccessKind = AccessKind::Exclusive;

    const SAFE: bool = true;

    fn index(inner: H, idx: usize) -> Self::ElementHandle {
        SoAElementPoint {
            idx,
            inner,
            _lt: PhantomData,
        }
    }
}

unsafe impl<'a, F, H, J, K, const N: usize> ProjectPlace<F>
    for SoAElementPoint<'a, H>
where
    F: PointField<N>,
    H: PlaceHandle<Target = SoAPoint<N>>,
    H: ProjectPlace<F::SoAField, Projected = J>,
    J: PlaceHandle<Target = [F::Target; N]>,
    [F::Target; N]:
        IndexPlace<usize, J, Lifetime<'a>, Instant, ElementHandle = K>,
    K: PlaceHandle<Target = F::Target>,
{
    type Projected = K;

    unsafe fn project_place(self, _subplace: F) -> K {
        let xs: H::Projected =
            unsafe { self.inner.project_place(F::SoAField::default()) };
        let x: K = IndexPlace::index(xs, self.idx);
        x
    }
}

// For fun, here is a valid "handle-monomorphized" version of the above impl.
#[cfg(false)]
unsafe impl<'a, F: PointField<N, Target: 'a>, const N: usize> ProjectPlace<F>
    for SoAElementPoint<'a, LocalHandle<SoAPoint<N>>>
{
    type Projected = LocalHandle<F::Target>;

    unsafe fn project_place(self, _subplace: F) -> LocalHandle<F::Target> {
        // SAFETY: Assume LocalHandle points to valid instance of SoAPoint.
        let xs: *mut [F::Target; N] = unsafe {
            self.inner
                .as_ptr()
                .byte_add(
                    F::SoAField::default()
                        .offset(self.inner.as_ptr().metadata())
                        .0,
                )
                .cast()
        };
        if self.idx >= N {
            panic!();
        }
        // SAFETY: We check that idx does not go past N. Since we know this is
        // a valid allocation, we know that add will not wrap.
        // This is unstable...
        // let ptr = unsafe { xs.as_mut_slice().as_mut_ptr().add(self.idx) };
        let ptr = unsafe { xs.cast::<F::Target>().add(self.idx) };
        // SAFETY: This handle can only be created through the IndexPlace impl
        // with exclusive access.
        unsafe { LocalHandle::new(ptr) }
    }
}

/// ```
/// let mut arr = SoAPoint { xs: [1, 3, 2], ys: [3, 2, 1] };
/// arr[2].y = 3;
/// assert_eq!(SoAPoint { xs: [1, 3, 2], ys: [3, 2, 3] }, arr);
/// ```
#[test]
fn write_through_local_handle() {
    let mut arr: SoAPoint<3> = SoAPoint {
        xs: [1, 3, 2],
        ys: [3, 2, 1],
    };
    unsafe {
        let hdl: LocalHandle<SoAPoint<3>> = LocalHandle::new(&raw mut arr);
        let hdl: SoAElementPoint<'_, LocalHandle<SoAPoint<3>>> =
            IndexPlace::index(hdl, 2);
        let hdl: LocalHandle<i32> =
            ProjectPlace::project_place(hdl, <field_of!(Point, y)>::default());
        WritePlace::write_place(hdl, 3);
    }
    assert_eq!(
        SoAPoint {
            xs: [1, 3, 2],
            ys: [3, 2, 3]
        },
        arr
    );
}

/// FIXME(tmandry): This passes miri. Will it ever pass the borrow checker?
/// ```
/// let mut arr = SoAPoint { xs: [1, 2, 3], ys: [3, 2, 1] };
/// let a = &mut arr[2].y;
/// let b = &mut arr[2].x;
/// let c = &mut arr[1].y;
/// *a = 3;
/// *b = 2;
/// *c = 1;
/// assert_eq!(SoAPoint { xs: [1, 2, 2], ys: [3, 1, 3] }, arr);
/// ```
#[test]
fn borrow_mut_overlapping() {
    let mut arr = SoAPoint {
        xs: [1, 2, 3],
        ys: [3, 2, 1],
    };
    let mut a: &mut i32 = unsafe {
        let hdl: LocalHandle<SoAPoint<3>> = LocalHandle::new(&raw mut arr);
        let hdl: SoAElementPoint<'_, LocalHandle<SoAPoint<3>>> =
            IndexPlace::index(hdl, 2);
        // TODO(tmandry): How do we get enough type information to know to
        // insert `field_of!(Point, y)` in the desugaring?
        let hdl: LocalHandle<i32> =
            ProjectPlace::project_place(hdl, <field_of!(Point, y)>::default());
        BorrowPlace::<&mut i32>::borrow(hdl)
    };
    let mut b: &mut i32 = unsafe {
        let hdl: LocalHandle<SoAPoint<3>> = LocalHandle::new(&raw mut arr);
        let hdl: SoAElementPoint<'_, LocalHandle<SoAPoint<3>>> =
            IndexPlace::index(hdl, 2);
        let hdl: LocalHandle<i32> =
            ProjectPlace::project_place(hdl, <field_of!(Point, x)>::default());
        BorrowPlace::<&mut i32>::borrow(hdl)
    };
    let mut c: &mut i32 = unsafe {
        let hdl: LocalHandle<SoAPoint<3>> = LocalHandle::new(&raw mut arr);
        let hdl: SoAElementPoint<'_, LocalHandle<SoAPoint<3>>> =
            IndexPlace::index(hdl, 1);
        let hdl: LocalHandle<i32> =
            ProjectPlace::project_place(hdl, <field_of!(Point, y)>::default());
        BorrowPlace::<&mut i32>::borrow(hdl)
    };
    unsafe {
        let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw mut a);
        let hdl: MutHandle<'_, i32> = DerefPlace::deref_place(hdl);
        WritePlace::write_place(hdl, 3);
    };
    unsafe {
        let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw mut b);
        let hdl: MutHandle<'_, i32> = DerefPlace::deref_place(hdl);
        WritePlace::write_place(hdl, 2);
    };
    unsafe {
        let hdl: LocalHandle<&mut i32> = LocalHandle::new(&raw mut c);
        let hdl: MutHandle<'_, i32> = DerefPlace::deref_place(hdl);
        WritePlace::write_place(hdl, 1);
    };
}

fn main() {}
