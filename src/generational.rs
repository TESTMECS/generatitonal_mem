/// A handle that stays valid until the variant’s generation changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle {
    pub generation: u32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum MyVariant {
    Int(i32),
    Text(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct GenVariant {
    inner: MyVariant,
    generation: u32,
}

#[allow(dead_code)]
impl GenVariant {
    pub fn new(inner: MyVariant) -> Self {
        Self {
            inner,
            generation: 0,
        }
    }

    /// Borrow a handle to current contents.
    pub fn handle(&self) -> Handle {
        Handle {
            generation: self.generation,
        }
    }

    /// Accessor that returns `Some(&MyVariant)` if still valid.
    pub fn get(&self, h: Handle) -> Option<&MyVariant> {
        (h.generation == self.generation).then(|| &self.inner)
    }

    /// Mutate to a different payload → bump generation, invalidating old handles.
    pub fn set(&mut self, new_inner: MyVariant) {
        self.inner = new_inner;
        self.generation = self.generation.wrapping_add(1);
    }
}
