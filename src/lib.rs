#!/bin/false

//! A map-like data structure that provides weak, light weight handles to inserted values.
//!
//! # Note
//! You should probably consider using the more widely used and battle tested
//! [slotmap crate](https://crates.io/crates/slotmap) rather than this one.

#![deny(clippy::pedantic)]

mod macros;

type IterItem<'a, T> = (Key, &'a T);
type IterMutItem<'a, T> = (Key, &'a mut T);
type IntoIterItem<T> = (Key, T);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Generation(pub u64);

/// A unique handle to a value in a slotmap.
/// ##### Memory use
/// The key is the size of a `u64` + `usize`, which is 16 bytes on 64 bit platforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    index: usize,
    generation: Generation,
}

use std::iter;
use std::slice;
use std::vec;

type IterFn<'a, T> = fn(&'a Item<T>) -> IterItem<T>;
type IterMutFn<'a, T> = fn(&'a mut Item<T>) -> IterMutItem<T>;
type IntoIterFn<T> = fn(Item<T>) -> IntoIterItem<T>;

pub struct Iter<'a, T>(iter::Map<slice::Iter<'a, Item<T>>, IterFn<'a, T>>);
pub struct IterMut<'a, T>(iter::Map<slice::IterMut<'a, Item<T>>, IterMutFn<'a, T>>);
pub struct IntoIter<T>(iter::Map<vec::IntoIter<Item<T>>, IntoIterFn<T>>);

struct Item<T> {
    value: T,
    key: Key,
}

#[derive(Clone, Copy)]
enum Slot {
    Occupied(usize),
    Vacant(Generation),
}

/// A slotmap that uses indirection for accesses to allow packing values next to each other.
///
/// # Performance
/// #### Access
/// Insertion, removal and access are still constant time operations like in `StandardSlotMap`.
/// Unlike `StandardSlotMap`, each of the previously mentioned operations
/// requires indexing into two vectors, the first vector index reads an element storing an
/// indirect index that points into the second vector, which is where the "indirection" name comes from!
///
/// Removing values doesn't require shifting any elements. This is done using
/// [`Vec::swap_remove`](Vec::swap_remove) interally and then updating the indirect indexes as needed.
/// Shrinking the underlying storage is not supported.
/// #### Iteration
/// All key value pairs are stored contigously in a vector, so iteration is as
/// fast as possible.
#[derive(Default)]
pub struct SlotMap<T> {
    items: Vec<Item<T>>,
    slots: Vec<Slot>,
    free: Vec<usize>,
}

impl Generation {
    pub fn next(self) -> Generation {
        Generation(self.0 + 1)
    }
}

impl Slot {
    pub fn unwrap_occupied(self) -> usize {
        match self {
            Slot::Occupied(i) => i,
            Slot::Vacant(_) => panic!(),
        }
    }
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

    /// see: [`StandardSlotMap::insert`](crate::StandardSlotMap::insert)
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

    /// see: [`StandardSlotMap::remove`](crate::StandardSlotMap::remove)
    #[allow(clippy::match_on_vec_items)]
    #[allow(clippy::missing_panics_doc)]
    pub fn remove(&mut self, key: Key) -> Option<T> {
        if self.get(key).is_some() {
            let indirect_index = self.slots[key.index].unwrap_occupied();
            self.free.push(indirect_index);
            self.slots[key.index] = Slot::Vacant(key.generation.next());
            if indirect_index == self.items.len() - 1 {
                self.items.pop().map(|i| i.value)
            } else {
                let last_item_index = self.items.last().unwrap().key.index;
                self.slots[last_item_index] = Slot::Occupied(indirect_index);
                Some(self.items.swap_remove(indirect_index).value)
            }
        } else {
            None
        }
    }

    /// see: [`StandardSlotMap::get`](crate::StandardSlotMap::get)
    #[must_use]
    pub fn get(&self, key: Key) -> Option<&T> {
        match self.slots.get(key.index).copied() {
            Some(Slot::Occupied(indirect_index))
                if self.items[indirect_index].key.generation == key.generation =>
            {
                Some(&self.items[indirect_index].value)
            }
            _ => None,
        }
    }

    /// see: [`StandardSlotMap::get_mut`](crate::StandardSlotMap::get_mut)
    #[must_use]
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        match self.slots.get(key.index).copied() {
            Some(Slot::Occupied(indirect_index))
                if self.items[indirect_index].key.generation == key.generation =>
            {
                Some(&mut self.items[indirect_index].value)
            }
            _ => None,
        }
    }

    /// see: [`StandardSlotMap::len`](crate::StandardSlotMap::len)
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// see: [`StandardSlotMap::is_empty`](crate::StandardSlotMap::is_empty)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }

    #[must_use]
    pub fn iter(&self) -> Iter<T> {
        Iter(
            self.items
                .as_slice()
                .iter()
                .map(|item| (item.key, &item.value)),
        )
    }
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(
            self.items
                .as_mut_slice()
                .iter_mut()
                .map(|item| (item.key, &mut item.value)),
        )
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
        IntoIter(self.items.into_iter().map(|item| (item.key, item.value)))
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
