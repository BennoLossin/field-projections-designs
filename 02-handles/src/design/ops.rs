use crate::design::{
    Metadata,
    borrowck::{AccessKind, Timing},
    subplace::Subplace,
};

pub trait ProxyPlace {
    type Handle: PlaceHandle;
}

pub trait PlaceHandle: Sized {
    type Target: ?Sized;
}

pub trait DerefHandle: ProxyPlace {
    const ACCESS: AccessKind;
    type Timing: Timing;

    unsafe fn handle_from_raw(this: *const Self) -> Self::Handle;
}

pub trait ReadPlace: PlaceHandle {
    const ACCESS: AccessKind;
    const SAFE: bool;

    unsafe fn read_place(self) -> Self::Target;
}

pub trait ReadMetadata: PlaceHandle {
    fn metadata(self) -> Metadata<Self::Target>;
}

pub trait MovePlace: ReadPlace {
    const ACCESS: AccessKind;
    const SAFE: bool;
}

pub trait WritePlace: PlaceHandle {
    const ACCESS: AccessKind;
    const SAFE: bool;

    unsafe fn write_place(self, value: Self::Target);
}

pub trait DropPlace: PlaceHandle {
    unsafe fn drop_place(self);
}

pub trait DropHusk: ProxyPlace {
    unsafe fn drop_husk(this: Self::Handle);
}

pub trait ProjectPlace<S>: PlaceHandle
where
    S: Subplace<Source = Self::Target>,
{
    type Projected: PlaceHandle<Target = S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected;
}

pub trait DerefPlace<PointeeTiming, PointerTiming>: PlaceHandle
where
    Self::Target: ProxyPlace,
    PointeeTiming: Timing,
    PointerTiming: Timing,
{
    const POINTEE_ACCESS: AccessKind;
    const POINTER_ACCESS: AccessKind;
    const SAFE: bool;

    unsafe fn deref_place(self) -> <Self::Target as ProxyPlace>::Handle;
}

pub trait BorrowPlace<Output>: PlaceHandle {
    const ACCESS: AccessKind;
    type Timing: Timing;
    const SAFE: bool;

    unsafe fn borrow(self) -> Output;
}

pub trait PlaceWrapper {
    type Inner: ?Sized;
}

pub unsafe trait WrapPlace<S>: PlaceWrapper
where
    S: Subplace<Source = Self::Inner>,
{
    type Wrapped: Subplace<Source = Self>;

    fn wrap(subplace: S) -> Self::Wrapped;
}
