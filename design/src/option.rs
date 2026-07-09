use crate::ops::place::{
    PlaceWrapper, WrapPlace,
    subplace::{Subplace, TransmutedSubplace},
};

impl<T> PlaceWrapper for Option<T> {
    type Inner = T;
}

unsafe impl<S> WrapPlace<S> for Option<S::Source>
where
    S: Subplace,
    S::Source: Sized,
    S::Target: Sized,
{
    type Wrapped = TransmutedSubplace<S, Option<S::Source>, S::Target>;

    fn wrap(subplace: S) -> Self::Wrapped {
        unsafe { TransmutedSubplace::new_unchecked(subplace) }
    }
}
