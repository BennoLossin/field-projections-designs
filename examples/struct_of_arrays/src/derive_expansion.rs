 struct SoAPoint<const N: usize> {
     x: [i32; N],
     y: [i32; N],
 }
 
 impl SoA for Point {
     type SoA<const N: usize> = SoAPoint<N>;
     type ArrayField<F: Field<Source = Self, Target: Sized>, const N: usize> =
         TransmutedField<F, SoAPoint<N>, [F::Target; N]>;
 
     fn array_field_from_struct<F, const N: usize>(
         _field: F,
     ) -> Self::ArrayField<F, N>
     where
         F: Field<Source = Self, Target: Sized>,
     {
         <TransmutedField<F, SoAPoint<N>, [F::Target; N]> as Default>::default()
     }
 }
 
 impl<const N: usize> Indexable<usize> for SoAPoint<N> {
     type Element = Point;
 }
 
 unsafe impl<H: PlaceHandle<Target = SoAPoint<N>>, const N: usize>
     IndexPlace<usize, H, Instant, Instant> for SoAPoint<N>
 {
     type ElementHandle = SoAHandle<Point, H, N>;
     const POINTEE_ACCESS: AccessKind = AccessKind::Shared;
     const POINTER_ACCESS: AccessKind = AccessKind::Shared;
     const SAFE: bool = true;
 
     fn index(handle: H, idx: usize) -> Self::ElementHandle {
         unsafe { SoAHandle::from_parts(handle, idx) }
     }
 }
