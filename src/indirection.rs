use crate::{Generation, Key};

struct Item<T> {
    value: T,
    key: Key,
}

enum Slot {
    Occupied(usize),
    Vacant(Generation),
}

#[derive(Default)]
pub struct SlotMap<T> {
    items: Vec<Item<T>>,
    slots: Vec<Slot>,
    free: Vec<usize>,
}

impl<T> SlotMap<T> {
    #[must_use]
    pub fn new() -> SlotMap<T> {
        SlotMap {
            items: Vec::new(),
            slots: Vec::new(),
            free: Vec::new(),
        }
    }

    #[must_use]
    #[allow(clippy::match_on_vec_items)]
    pub fn insert(&mut self, value: T) -> Key {
        if let Some(index) = self.free.pop() {
            match self.slots[index] {
                Slot::Vacant(generation) => {
                    let key = Key { index, generation };
                    self.items.push(Item { value, key });
                    self.slots[index] = Slot::Occupied(self.items.len() - 1);
                    key
                }
                Slot::Occupied(_) => unreachable!(),
            }
        } else {
            let key = Key {
                index: self.slots.len(),
                generation: Generation(0),
            };
            self.items.push(Item { value, key });
            self.slots.push(Slot::Occupied(self.items.len() - 1));
            key
        }
    }

    #[must_use]
    pub fn get(&self, key: Key) -> Option<&T> {
        match self.slots.get(key.index) {
            Some(Slot::Occupied(indirect_index))
                if self.items[key.index].key.generation == key.generation =>
            {
                Some(&self.items[key.index].value)
            }
            _ => None,
        }
    }
    #[must_use]
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        match self.slots.get(key.index) {
            Some(Slot::Occupied(indirect_index))
                if self.items[key.index].key.generation == key.generation =>
            {
                Some(&mut self.items[key.index].value)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::macros::test_insert_get!(SlotMap<_>);
}
