use std::mem;
use std::ops::{Index, IndexMut};

use crate::{IntoIterItem, IterItem, IterMutItem, Key};

pub struct Iter<'a, T>(::std::iter::Enumerate<::std::slice::Iter<'a, Slot<T>>>);

pub struct IterMut<'a, T>(::std::iter::Enumerate<::std::slice::IterMut<'a, Slot<T>>>);

pub struct IntoIter<T>(::std::iter::Enumerate<::std::vec::IntoIter<Slot<T>>>);

#[derive(Clone, Debug)]
enum Slot<T> {
    Occupied(u64, T),
    Vacant(u64),
}

#[derive(Clone, Default)]
pub struct StandardSlotMap<T> {
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
}

impl<T> StandardSlotMap<T> {
    pub fn new() -> StandardSlotMap<T> {
        StandardSlotMap {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> Key {
        if let Some(index) = self.free.pop() {
            match self.slots[index] {
                Slot::Vacant(generation) => {
                    let new_slot = Slot::Occupied(generation, value);
                    let old_slot = mem::replace(&mut self.slots[index], new_slot);
                    mem::drop(old_slot);
                    Key { index, generation }
                }
                _ => unreachable!(),
            }
        } else {
            self.slots.push(Slot::Occupied(0, value));
            Key {
                index: self.slots.len() - 1,
                generation: 0,
            }
        }
    }

    pub fn remove(&mut self, key: Key) -> Option<T> {
        if self.get(key).is_some() {
            self.free.push(key.index);
            let new_slot = Slot::Vacant(key.generation + 1);
            let old_slot = mem::replace(&mut self.slots[key.index], new_slot);
            match old_slot {
                Slot::Occupied(_, value) => Some(value),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    pub fn get(&self, key: Key) -> Option<&T> {
        match self.slots.get(key.index) {
            Some(Slot::Occupied(generation, item)) if *generation == key.generation => Some(item),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        match self.slots.get_mut(key.index) {
            Some(Slot::Occupied(generation, value)) if *generation == key.generation => Some(value),
            _ => None,
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter(self.slots.as_slice().iter().enumerate())
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.slots.as_mut_slice().iter_mut().enumerate())
    }

    pub fn len(&self) -> usize {
        self.slots.len() - self.free.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Index<Key> for StandardSlotMap<T> {
    type Output = T;

    fn index(&self, key: Key) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<T> IndexMut<Key> for StandardSlotMap<T> {
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

impl<'a, T> IntoIterator for &'a StandardSlotMap<T> {
    type Item = IterItem<'a, T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut StandardSlotMap<T> {
    type Item = IterMutItem<'a, T>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for StandardSlotMap<T> {
    type Item = IntoIterItem<T>;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.slots.into_iter().enumerate())
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = IterItem<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.find_map(|(index, slot)| match slot {
            Slot::Occupied(generation, value) => Some((
                Key {
                    index,
                    generation: *generation,
                },
                value,
            )),
            _ => None,
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = IterMutItem<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.find_map(|(index, slot)| match slot {
            Slot::Occupied(generation, value) => Some((
                Key {
                    index,
                    generation: *generation,
                },
                value,
            )),
            _ => None,
        })
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = IntoIterItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.find_map(|(index, slot)| match slot {
            Slot::Occupied(generation, value) => Some((Key { index, generation }, value)),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::macros::test_insert_get!(StandardSlotMap<_>);
    crate::macros::test_remove!(StandardSlotMap<_>);
    crate::macros::test_len!(StandardSlotMap<_>);
    crate::macros::test_uaf!(StandardSlotMap<_>);
    crate::macros::test_iterator!(StandardSlotMap<_>);
    crate::macros::test_iterator_skip_vacant!(StandardSlotMap<_>);

    #[test]
    fn test_slot_reuse() {
        let mut slotmap = StandardSlotMap::new();
        let a = slotmap.insert(());
        let b = slotmap.insert(());
        let c = slotmap.insert(());
        slotmap.remove(a);
        slotmap.remove(b);
        slotmap.remove(c);
        slotmap.insert(());
        slotmap.insert(());
        slotmap.insert(());
        slotmap.insert(());
        assert_eq!(slotmap.len(), 4);
    }
}
