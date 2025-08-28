use std::mem;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle {
    idx: usize,
    gen: u32,
}

#[derive(Debug)]
struct Slot<T> {
    gen: u32,
    val: Option<T>,
}

#[derive(Debug, Default)]
pub struct DynVec<T> {
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
}

impl<T> DynVec<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, value: T) -> Handle {
        if let Some(idx) = self.free.pop() {
            let gen = self.slots[idx].gen;
            self.slots[idx].val = Some(value);
            Handle { idx, gen }
        } else {
            let idx = self.slots.len();
            self.slots.push(Slot {
                gen: 0,
                val: Some(value),
            });
            Handle { idx, gen: 0 }
        }
    }

    /// Reassigns the slot (e.g., "Variant changed type"): bumps generation.
    pub fn replace(&mut self, h: Handle, value: T) -> Result<(), ()> {
        let slot = self.slots.get_mut(h.idx).ok_or(())?;
        if slot.gen != h.gen || slot.val.is_none() {
            return Err(());
        }
        slot.gen = slot.gen.wrapping_add(1);
        slot.val = Some(value);
        Ok(())
    }

    pub fn get(&self, h: Handle) -> Option<&T> {
        let slot = self.slots.get(h.idx)?;
        (slot.gen == h.gen).then(|| slot.val.as_ref()).flatten()
    }

    pub fn get_mut(&mut self, h: Handle) -> Option<&mut T> {
        let slot = self.slots.get_mut(h.idx)?;
        (slot.gen == h.gen).then(|| slot.val.as_mut()).flatten()
    }

    /// Deletes the item: bumps generation and frees the slot.
    pub fn remove(&mut self, h: Handle) -> Option<T> {
        let slot = self.slots.get_mut(h.idx)?;
        if slot.gen != h.gen || slot.val.is_none() {
            return None;
        }
        let old = slot.val.take();
        slot.gen = slot.gen.wrapping_add(1);
        self.free.push(h.idx);
        old
    }

    /// Bulk mutation (e.g., clear or reallocate): invalidate *all* contents.
    pub fn clear(&mut self) {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.val.is_some() {
                slot.val = None;
                slot.gen = slot.gen.wrapping_add(1);
                self.free.push(i);
            }
        }
    }

    /// Swap-without-borrowing-T: contents remain valid (no gen bump).
    pub fn swap(&mut self, a: Handle, b: Handle) -> Result<(), ()> {
        let (sa, sb) = {
            let sa = self.slots.get(a.idx).ok_or(())?;
            let sb = self.slots.get(b.idx).ok_or(())?;
            if sa.gen != a.gen || sb.gen != b.gen {
                return Err(());
            }
            (a.idx, b.idx)
        };
        self.slots.swap(sa, sb);
        Ok(())
    }

    /// "Type change" helper for Variant-like containers.
    pub fn map_invalidate<F>(&mut self, h: Handle, f: F) -> Result<(), ()>
    where
        F: FnOnce(Option<T>) -> Option<T>,
    {
        let slot = self.slots.get_mut(h.idx).ok_or(())?;
        if slot.gen != h.gen {
            return Err(());
        }
        let new_val = f(mem::take(&mut slot.val));
        // Changing contents' identity => bump gen
        slot.gen = slot.gen.wrapping_add(1);
        slot.val = new_val;
        Ok(())
    }
}
