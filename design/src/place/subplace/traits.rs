use crate::ptr::Metadata;

/// Represents a *subplace* in `Self::Source`.
///
/// A subplace is directly contained in its parent (i.e. there is no pointer
/// indirection).
pub unsafe trait Subplace: Sized {
    type Source: ?Sized;
    type Target: ?Sized;

    fn offset(
        self,
        metadata: Metadata<Self::Source>,
    ) -> (usize, Metadata<Self::Target>);
}

/// A type whose values can be `match`ed.
///
/// For every value `VARIANT` in `Self::VARIANTS`, `Self` implements
/// `HasVariant<VARIANT>`. Casting a pointer to `Self` to `<Self as
/// HasVariant<VARIANTS>>::VariantType` is sound when `variant_at` returned
/// `VARIANT`.
pub unsafe trait Matchable {
    const VARIANTS: &'static [&'static str];

    unsafe fn variant_at(ptr: *const Self) -> &'static str;
}

pub unsafe trait HasVariant<const VARIANT: &'static str>:
    Matchable
{
    type VariantType;
}

pub type VariantType<E, const VARIANT: &'static str> =
    <E as HasVariant<VARIANT>>::VariantType;
