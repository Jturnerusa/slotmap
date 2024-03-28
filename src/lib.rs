#!/bin/false

//! A map-like data structure that provides weak, light weight handles to inserted values.
//!
//! # Note
//! You should probably consider using the more widely used and battle tested
//! [slotmap crate](https://crates.io/crates/slotmap) rather than this one.

#![deny(clippy::pedantic)]

use std::iter;
use std::slice;
use std::vec;

/// A unique handle to a value in a slotmap.
/// ##### Memory use
/// The key is the size of a `u64` + `usize`, which is 16 bytes on 64 bit platforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    index: usize,
    generation: Generation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Generation(pub u64);

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
}
