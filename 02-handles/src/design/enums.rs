pub unsafe trait Matchable {
    const VARIANTS: &'static [&'static str];

    unsafe fn variant_at(ptr: *const Self) -> &'static str;
}

pub trait HasVariant<const VARIANT: &'static str>: Matchable {
    type VariantType;
}

pub type VariantType<E, const VARIANT: &'static str> = <E as HasVariant<VARIANT>>::VariantType;
