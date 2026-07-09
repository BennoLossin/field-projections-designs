use std::ptr::NonNull;

use crate::design::ops::{PlaceHandle, ProxyPlace};

impl<T> ProxyPlace for Vec<T> {
    type Handle = VecHandle<T>;
}

pub struct VecHandle<T> {
    ptr: NonNull<[T]>,
}

impl<T> PlaceHandle for VecHandle<T> {
    type Target = [T];
}
