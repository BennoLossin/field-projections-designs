use crate::ops::place::{
    DerefHandle,
    DerefPlace,
    PlaceHandle,
    ProjectPlace,
    ProxyPlace,
    ReadPlace,
    ReadVariant,
    VariantPlace,
    borrowck::{
        AccessKind,
        Instant,
    },
    subplace::{
        HasVariant,
        Matchable,
        Subplace,
        VariantType,
    },
};

pub struct LocalHandle<T: ?Sized> {
    ptr: *const T,
}

impl<T: ?Sized> Clone for LocalHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ?Sized> Copy for LocalHandle<T> {}

impl<T: ?Sized> LocalHandle<T> {
    pub unsafe fn new(ptr: *const T) -> Self {
        Self { ptr }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.cast_mut()
    }
}

impl<T: ?Sized> PlaceHandle for LocalHandle<T> {
    type Target = T;
}

impl<P> DerefPlace<P::Timing, Instant> for LocalHandle<P>
where
    P: DerefHandle,
{
    const POINTEE_ACCESS: AccessKind = P::ACCESS;
    const POINTER_ACCESS: AccessKind = P::ACCESS;

    const SAFE: bool = true;

    unsafe fn deref_place(self) -> <Self::Target as ProxyPlace>::Handle {
        unsafe { P::handle_from_raw(self.as_ptr()) }
    }
}

impl<T: ?Sized + Matchable> ReadVariant for LocalHandle<T> {
    unsafe fn read_variant(self) -> &'static str {
        unsafe { T::variant_at(self.ptr) }
    }
}

impl<T, const VARIANT: &'static str> VariantPlace<VARIANT> for LocalHandle<T>
where
    T: ?Sized + HasVariant<VARIANT>,
{
    type ToVariant = LocalHandle<VariantType<T, VARIANT>>;

    unsafe fn cast(self) -> Self::ToVariant {
        LocalHandle { ptr: self.ptr.cast() }
    }
}

impl<S> ProjectPlace<S> for LocalHandle<S::Source>
where
    S: Subplace,
{
    type Projected = LocalHandle<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        LocalHandle {
            ptr: unsafe { self.ptr.project_place(subplace) },
        }
    }
}

impl<T> ReadPlace for LocalHandle<T> {
    const ACCESS: AccessKind = AccessKind::Shared;
    const SAFE: bool = true;

    unsafe fn read_place(self) -> Self::Target {
        unsafe { self.ptr.read() }
    }
}
