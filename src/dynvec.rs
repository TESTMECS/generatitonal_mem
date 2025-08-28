/// Give each element a generation counter. A handle is just {index, generation}.
/// Any operation that might invalidate contents (remove, replace-with-different-type, clear, compaction) bumps the generation. Using a handle after that fails to upgrade.
use std::mem;

/// A handle to a slot in the vector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle {
    pub idx: usize,
    pub generation: u32,
}

/// A vector of elements with generational semantics.
#[derive(Debug)]
struct Slot<T> {
    generation: u32,
    val: Option<T>,
}

/// Free contains a list of indices of slots that are free.
/// The free list is a vector of indices, so that we can use Vec::swap_remove.
#[derive(Debug)]
pub struct DynVec<T> {
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
}

/// Initalize a DynVec with a default value.
impl<T> Default for DynVec<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }
}

impl<T> DynVec<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new item: bumps generation and returns a handle.
    pub fn insert(&mut self, value: T) -> Handle {
        if let Some(idx) = self.free.pop() {
            // reuse a slot
            let generation = self.slots[idx].generation; // save the old generation
            self.slots[idx].val = Some(value); // set the new value
            Handle { idx, generation } // return the updated handle
        } else {
            // no free slots, so we need to add a new slot
            let idx = self.slots.len(); // get the index of the new slot
            self.slots.push(Slot {
                generation: 0,
                val: Some(value),
            });
            Handle { idx, generation: 0 }
        }
    }

    /// Reassigns the slot (e.g., "Variant changed type"): bumps generation.
    pub fn replace(&mut self, h: Handle, value: T) -> Result<Handle, ()> {
        let slot = self.slots.get_mut(h.idx).ok_or(())?; // get a mutable reference to the slot
        if slot.generation != h.generation || slot.val.is_none() {
            return Err(()); // generation mismatch or slot is not initalized
        }
        slot.generation = slot.generation.wrapping_add(1); // bump the generation
        slot.val = Some(value);
        Ok(Handle {
            idx: h.idx,
            generation: slot.generation,
        })
    }

    /// Get a reference to the value of the slot.
    pub fn get(&self, h: Handle) -> Option<&T> {
        let slot = self.slots.get(h.idx)?;
        (slot.generation == h.generation)
            .then(|| slot.val.as_ref())
            .flatten()
    }

    /// Get a mutable reference to the value of the slot.
    pub fn get_mut(&mut self, h: Handle) -> Option<&mut T> {
        let slot = self.slots.get_mut(h.idx)?;
        (slot.generation == h.generation)
            .then(|| slot.val.as_mut())
            .flatten()
    }

    #[allow(dead_code)]
    /// Deletes the item: bumps generation and frees the slot.
    pub fn remove(&mut self, h: Handle) -> Option<T> {
        let slot = self.slots.get_mut(h.idx)?;
        if slot.generation != h.generation || slot.val.is_none() {
            return None;
        }
        let old = slot.val.take();
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(h.idx);
        old
    }

    #[allow(dead_code)]
    /// Bulk mutation (e.g., clear or reallocate): invalidate *all* contents.
    pub fn clear(&mut self) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.val.is_some() {
                slot.val = None;
                slot.generation = slot.generation.wrapping_add(1);
                self.free.push(i);
            }
        }
    }

    #[allow(dead_code)]
    /// Swap-without-borrowing-T: contents remain valid (no gen bump).
    pub fn swap(&mut self, a: Handle, b: Handle) -> Result<(), ()> {
        let (sa, sb) = {
            let sa = self.slots.get(a.idx).ok_or(())?;
            let sb = self.slots.get(b.idx).ok_or(())?;
            if sa.generation != a.generation || sb.generation != b.generation {
                return Err(());
            }
            (a.idx, b.idx)
        };
        self.slots.swap(sa, sb);
        Ok(())
    }

    #[allow(dead_code)]
    /// "Type change" helper for Variant-like containers.
    pub fn map_invalidate<F>(&mut self, h: Handle, f: F) -> Result<(), ()>
    where
        F: FnOnce(Option<T>) -> Option<T>,
    {
        let slot = self.slots.get_mut(h.idx).ok_or(())?;
        if slot.generation != h.generation {
            return Err(());
        }
        let new_val = f(mem::take(&mut slot.val));
        // Changing contents' identity => bump gen
        slot.generation = slot.generation.wrapping_add(1);
        slot.val = new_val;
        Ok(())
    }

    /// Get the number of slots (for debugging/testing purposes)
    pub fn len(&self) -> usize {
        self.slots.len()
    }
}
