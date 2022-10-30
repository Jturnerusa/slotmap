use std::iter;
use std::mem;
use std::ops::{Index, IndexMut};
use std::slice;
use std::vec;

use crate::{IntoIterItem, IterItem, IterMutItem, Key};

type EnumeratedFilterMap<T, F> = iter::FilterMap<iter::Enumerate<T>, F>;
type IterFn<'a, T> = fn((usize, &'a Slot<T>)) -> Option<IterItem<'a, T>>;
type IterMutFn<'a, T> = fn((usize, &'a mut Slot<T>)) -> Option<IterMutItem<'a, T>>;
type IntoIterFn<T> = fn((usize, Slot<T>)) -> Option<IntoIterItem<T>>;

pub struct Iter<'a, T>(EnumeratedFilterMap<slice::Iter<'a, Slot<T>>, IterFn<'a, T>>);
pub struct IterMut<'a, T>(EnumeratedFilterMap<slice::IterMut<'a, Slot<T>>, IterMutFn<'a, T>>);
pub struct IntoIter<T>(EnumeratedFilterMap<vec::IntoIter<Slot<T>>, IntoIterFn<T>>);

#[derive(Clone, Debug)]
enum Slot<T> {
    Occupied(u64, T),
    Vacant(u64),
}

#[derive(Clone, Default)]
pub struct SlotMap<T> {
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
}

impl<T> SlotMap<T> {
    #[must_use]
    pub fn new() -> SlotMap<T> {
        SlotMap {
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
                    self.slots[index] = Slot::Occupied(generation, value);
                    Key { index, generation }
                }
                Slot::Occupied(..) => unreachable!(),
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
            match mem::replace(&mut self.slots[key.index], Slot::Vacant(key.generation + 1)) {
                Slot::Occupied(_, value) => Some(value),
                Slot::Vacant(_) => unreachable!(),
            }
        } else {
            None
        }
    }

    #[must_use]
    pub fn get(&self, key: Key) -> Option<&T> {
        match self.slots.get(key.index) {
            Some(Slot::Occupied(generation, item)) if *generation == key.generation => Some(item),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        match self.slots.get_mut(key.index) {
            Some(Slot::Occupied(generation, value)) if *generation == key.generation => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter(self.slots.as_slice().iter().enumerate().filter_map(
            |(index, slot): (usize, &'a Slot<T>)| match slot {
                Slot::Occupied(generation, value) => {
                    let key = Key {
                        index,
                        generation: *generation,
                    };
                    Some((key, value))
                }
                Slot::Vacant(_) => None,
            },
        ))
    }

    #[must_use]
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut(self.slots.as_mut_slice().iter_mut().enumerate().filter_map(
            |(index, slot): (usize, &'a mut Slot<T>)| match slot {
                Slot::Occupied(generation, value) => {
                    let key = Key {
                        index,
                        generation: *generation,
                    };
                    Some((key, value))
                }
                Slot::Vacant(_) => None,
            },
        ))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.slots.len() - self.free.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Index<Key> for SlotMap<T> {
    type Output = T;

    fn index(&self, key: Key) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<T> IndexMut<Key> for SlotMap<T> {
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

impl<'a, T> IntoIterator for &'a SlotMap<T> {
    type Item = IterItem<'a, T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SlotMap<T> {
    type Item = IterMutItem<'a, T>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for SlotMap<T> {
    type Item = IntoIterItem<T>;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.slots.into_iter().enumerate().filter_map(
            |(index, slot): (usize, Slot<T>)| match slot {
                Slot::Occupied(generation, value) => {
                    let key = Key { index, generation };
                    Some((key, value))
                }
                Slot::Vacant(_) => None,
            },
        ))
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = IterItem<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = IterMutItem<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = IntoIterItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::macros::test_insert_get!(SlotMap<_>);
    crate::macros::test_remove!(SlotMap<_>);
    crate::macros::test_len!(SlotMap<_>);
    crate::macros::test_uaf!(SlotMap<_>);
    crate::macros::test_iterator!(SlotMap<_>);
    crate::macros::test_iterator_skip_vacant!(SlotMap<_>);
    crate::macros::test_double_ended_iterator!(SlotMap<_>);

    #[test]
    #[allow(unused_must_use)]
    fn test_slot_reuse() {
        let mut slotmap = SlotMap::new();
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
