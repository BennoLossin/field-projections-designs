use std::{
    marker::{
        PhantomData,
        PhantomInvariant,
    },
    ptr::NonNull,
};

use design::{
    ops::place::{
        BorrowPlace,
        CreateHandle,
        PlaceHandle,
        PlaceProxy,
        ProjectPlace,
        ReadPlace,
        borrowck::{
            AccessKind,
            Instant,
            Lifetime,
        },
    },
    place::{
        ArrayIndex,
        Field,
    },
};

pub trait SoA: Sized {
    type SoA<const N: usize>;

    type ArrayField<F, const N: usize>: Field<Source = Self::SoA<N>, Target = [F::Target; N]>
    where
        F: Field<Source = Self, Target: Sized>;

    fn array_field_from_struct<F, const N: usize>(
        field: F,
    ) -> Self::ArrayField<F, N>
    where
        F: Field<Source = Self, Target: Sized>;
}

pub struct SoARef<'a, T: SoA, const N: usize> {
    soa: NonNull<T::SoA<N>>,
    idx: usize,
    _lt: PhantomData<&'a T>,
}

pub struct SoAMut<'a, T: SoA, const N: usize> {
    soa: NonNull<T::SoA<N>>,
    idx: usize,
    _lt: PhantomData<&'a mut T>,
}

impl<T: SoA, const N: usize> PlaceProxy for SoARef<'_, T, N> {
    type Target = T;
}

impl<T: SoA, const N: usize> PlaceProxy for SoAMut<'_, T, N> {
    type Target = T;
}

unsafe impl<'a, T, const N: usize> CreateHandle<Lifetime<'a>>
    for SoARef<'a, T, N>
where
    T: SoA,
{
    type Handle = SoAHandle<T, NonNull<T::SoA<N>>, N>;
    const ACCESS: AccessKind = AccessKind::Shared;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        SoAHandle {
            handle: unsafe { (*this).soa },
            idx: unsafe { (*this).idx },
            _soa: PhantomInvariant::new(),
        }
    }
}

unsafe impl<'a, T, const N: usize> CreateHandle<Lifetime<'a>>
    for SoAMut<'a, T, N>
where
    T: SoA,
{
    type Handle = SoAHandle<T, NonNull<T::SoA<N>>, N>;
    const ACCESS: AccessKind = AccessKind::Exclusive;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle {
        SoAHandle {
            handle: unsafe { (*this).soa },
            idx: unsafe { (*this).idx },
            _soa: PhantomInvariant::new(),
        }
    }
}

pub struct SoAHandle<T, H, const N: usize> {
    handle: H,
    idx: usize,
    _soa: PhantomInvariant<T>,
}

impl<T, H, const N: usize> SoAHandle<T, H, N> {
    pub unsafe fn from_parts(handle: H, idx: usize) -> Self {
        Self {
            handle,
            idx,
            _soa: PhantomInvariant::new(),
        }
    }

    pub unsafe fn into_parts(self) -> (H, usize) {
        let Self { handle, idx, _soa } = self;
        (handle, idx)
    }
}

impl<const N: usize, H, T> PlaceHandle for SoAHandle<T, H, N>
where
    T: SoA,
    H: PlaceHandle<Target = T::SoA<N>>,
{
    type Target = T;
}

unsafe impl<T, S, H, const N: usize> ProjectPlace<S> for SoAHandle<T, H, N>
where
    T: SoA,
    H: PlaceHandle<Target = T::SoA<N>>,
    S: Field<Source = T, Target: Sized>,
    H: ProjectPlace<T::ArrayField<S, N>>,
    H::Projected: ProjectPlace<ArrayIndex<S::Target, N>>,
{
    type Projected =
        <H::Projected as ProjectPlace<ArrayIndex<S::Target, N>>>::Projected;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let (hdl, idx) = unsafe { self.into_parts() };
        unsafe {
            hdl.project_place(T::array_field_from_struct(subplace))
                .project_place(ArrayIndex::new_unchecked(idx))
        }
    }
}

unsafe impl<T, H, const N: usize> ReadPlace for SoAHandle<T, H, N>
where
    T: SoA,
    H: PlaceHandle<Target = T::SoA<N>>,
{
    const ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        todo!()
    }
}

unsafe impl<'a, T, H, const N: usize> BorrowPlace<SoARef<'a, T, N>>
    for SoAHandle<T, H, N>
where
    T: SoA,
    H: PlaceHandle<Target = T::SoA<N>>
        // TODO: this is actually not fully sound... we probably need something
        // different here...
        + BorrowPlace<NonNull<T::SoA<N>>, Timing = Instant>,
{
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> SoARef<'a, T, N> {
        let Self { handle, idx, _soa } = self;
        SoARef {
            soa: unsafe { handle.borrow() },
            idx,
            _lt: PhantomData,
        }
    }
}

unsafe impl<'a, T, H, const N: usize> BorrowPlace<SoAMut<'a, T, N>>
    for SoAHandle<T, H, N>
where
    T: SoA,
    H: PlaceHandle<Target = T::SoA<N>>
        // TODO: this is actually not fully sound... we probably need something
        // different here...
        + BorrowPlace<NonNull<T::SoA<N>>, Timing = Instant>,
{
    const ACCESS: AccessKind = AccessKind::Shared;
    type Timing = Lifetime<'a>;
    const SAFE: bool = true;

    unsafe fn borrow(self) -> SoAMut<'a, T, N> {
        let Self { handle, idx, _soa } = self;
        SoAMut {
            soa: unsafe { handle.borrow() },
            idx,
            _lt: PhantomData,
        }
    }
}
