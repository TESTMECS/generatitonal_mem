use std::fmt;

/// A handle that stays valid until the variant’s generation changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle {
    gen: u32,
}

#[derive(Debug)]
pub enum MyVariant {
    Int(i32),
    Text(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct GenVariant {
    inner: MyVariant,
    gen: u32,
}

impl GenVariant {
    pub fn new(inner: MyVariant) -> Self {
        Self { inner, gen: 0 }
    }

    /// Borrow a handle to current contents.
    pub fn handle(&self) -> Handle {
        Handle { gen: self.gen }
    }

    /// Accessor that returns `Some(&MyVariant)` if still valid.
    pub fn get(&self, h: Handle) -> Option<&MyVariant> {
        (h.gen == self.gen).then(|| &self.inner)
    }

    /// Mutate to a different payload → bump generation, invalidating old handles.
    pub fn set(&mut self, new_inner: MyVariant) {
        self.inner = new_inner;
        self.gen = self.gen.wrapping_add(1);
    }
}
