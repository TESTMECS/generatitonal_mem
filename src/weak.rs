use std::ops::Deref;
use crate::dynvec::{DynVec, Handle};

pub struct Elem<'a, T> {
    pub parent: &'a DynVec<T>,
    pub handle: Handle,
}

impl<'a, T> Elem<'a, T> {
    pub fn new(parent: &'a DynVec<T>, handle: Handle) -> Option<Self> {
        parent.get(handle).map(|_| Self { parent, handle })
    }
}

impl<'a, T> Deref for Elem<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.parent
            .get(self.handle)
            .expect("use-after-invalidate: element no longer valid")
    }
}
