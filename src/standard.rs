use std::iter;
use std::mem;
use std::ops::{Index, IndexMut};
use std::slice;
use std::vec;

use crate::{Generation, IntoIterItem, IterItem, IterMutItem, Key};

type EnumeratedFilterMap<T, F> = iter::FilterMap<iter::Enumerate<T>, F>;
type IterFn<'a, T> = fn((usize, &'a Slot<T>)) -> Option<IterItem<'a, T>>;
type IterMutFn<'a, T> = fn((usize, &'a mut Slot<T>)) -> Option<IterMutItem<'a, T>>;
type IntoIterFn<T> = fn((usize, Slot<T>)) -> Option<IntoIterItem<T>>;

pub struct Iter<'a, T>(EnumeratedFilterMap<slice::Iter<'a, Slot<T>>, IterFn<'a, T>>);
pub struct IterMut<'a, T>(EnumeratedFilterMap<slice::IterMut<'a, Slot<T>>, IterMutFn<'a, T>>);
pub struct IntoIter<T>(EnumeratedFilterMap<vec::IntoIter<Slot<T>>, IntoIterFn<T>>);

#[derive(Clone, Debug)]
enum Slot<T> {
    Occupied(Generation, T),
    Vacant(Generation),
}

/// A simple and performant slotmap implemented with a simple vector of slots.
/// # Performance
/// Insertion, removal and access are all constant time operations that are roughly as
/// fast as indexing a vector.
///
/// Removing values does not require shifting elements like in a vector because slots
/// are reused. Shrinking the underlying storage is not supported.
///
/// Iteration requires scanning the entire underlying storage in order to skip over vacant
/// slots. Slotmaps with lots of removed values may have large gaps of vacant slots that
/// slow down interation. If you need very fast iteration and can tolerate a single layer of
/// indirection when accessing values you may want to consider using [todo!](todo)

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

    /// Inserts a value into the slotmap. This returns a unique [key](`crate::Key`) that can
    /// later be be used to [access](Self::get) and [remove](Self::remove) values.
    ///
    /// Insert will reuse vacant slots when they are available.
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
            let generation = Generation(0);
            self.slots.push(Slot::Occupied(generation, value));
            Key {
                index: self.slots.len() - 1,
                generation,
            }
        }
    }

    /// Removes the value associated with [key](Key) from the slotmap.
    /// This will return [None](None) if provided with a stale [key](Key).
    pub fn remove(&mut self, key: Key) -> Option<T> {
        if self.get(key).is_some() {
            self.free.push(key.index);
            match mem::replace(
                &mut self.slots[key.index],
                Slot::Vacant(key.generation.next()),
            ) {
                Slot::Occupied(_, value) => Some(value),
                Slot::Vacant(_) => unreachable!(),
            }
        } else {
            None
        }
    }

    /// Returns a shared reference to the value associated with the [key](Key).
    /// Attempting to retrive a value that has been removed will return [None](None).
    /// This method should be used instead of [index](Self::index) if you aren't sure
    /// if a value still exists.
    #[must_use]
    pub fn get(&self, key: Key) -> Option<&T> {
        match self.slots.get(key.index) {
            Some(Slot::Occupied(generation, item)) if *generation == key.generation => Some(item),
            _ => None,
        }
    }

    /// Returns an exclusive reference to the value associated with the [key](Key) and
    /// otherwise behaves indentically to [get](Self::get).
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

    /// Returns the number of occupied slots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.slots.len() - self.free.len()
    }

    /// Returns true if there are no occupied slots.
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
