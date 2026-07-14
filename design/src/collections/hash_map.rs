use std::{
    collections::HashMap,
    marker::PhantomData,
};

use crate::ops::place::{
    IndexPlace,
    Indexable,
    PlaceHandle,
};

impl<K, V> Indexable<K> for HashMap<K, V> {
    type Element = V;
}

pub struct HashMapEntryHandle<H, K, V> {
    key: K,
    handle: H,
    _value: PhantomData<V>,
}

impl<H, K, V> PlaceHandle for HashMapEntryHandle<H, K, V> {
    type Target = V;
}

/*
unsafe impl<K, H, V> IndexPlace<K, H> for HashMap<K, V>
where
    H: PlaceHandle<Target = Self>,
{
    type ElementHandle = HashMapEntryHandle<H, K, V>;

    fn index(handle: H, key: K) -> Self::ElementHandle {
        HashMapEntryHandle {
            key,
            handle,
            _value: PhantomData,
        }
    }
}
*/

pub struct HashMapRefHandle<'q, H, Q, K, V> {
    key: &'q Q,
    handle: H,
    _key: PhantomData<K>,
    _value: PhantomData<V>,
}

impl<'q, H, Q, K, V> PlaceHandle for HashMapRefHandle<'q, H, Q, K, V> {
    type Target = V;
}

unsafe impl<'q, H, Q, K, V> IndexPlace<&'q Q, H> for HashMap<K, V>
where
    H: PlaceHandle<Target = Self>,
{
    type ElementHandle = HashMapRefHandle<'q, H, Q, K, V>;

    fn index(handle: H, key: &'q Q) -> Self::ElementHandle {
        HashMapRefHandle {
            key,
            handle,
            _key: PhantomData,
            _value: PhantomData,
        }
    }
}
