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

    /// Inserts a value into the slotmap. This returns a unique key that can
    /// later be be used to access and remove values.
    /// ##### Performance
    /// Insertion should be roughly as fast as inserting into into a vector,
    /// or if there are no empty slots pushing onto the end.
    /// ##### Slot reuse
    /// Insert will reuse vacant slots when they are available similar to an
    /// arena.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    /// let key = slotmap.insert("an example value");
    /// ```
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

    /// Removes the value associated with a key from the slotmap.
    /// This will return `None` if provided with a stale key.
    /// ##### Performance
    /// Removing values is roughly as fast as mutating an element
    /// of a vector. Removing values does not require shifting elements,
    /// they just get marked as vacant to allow reusing them later.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    /// let key = slotmap.insert("an example value");
    /// assert!(matches!(
    ///     slotmap.remove(key),
    ///     Some("an example value")
    /// ));
    /// assert!(slotmap.remove(key).is_none());
    /// ```
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

    /// Returns a shared reference to the value associated with the key.
    /// Attempting to retrive a value that has been removed will return `None`.
    /// This method should be used instead of indexing if you aren't sure that
    /// if a value still exists.
    /// ##### Performance
    /// Accessing elements should be roughly as fast as indexing a vector.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    /// let key = slotmap.insert("an example value");
    /// assert!(matches!(
    ///     slotmap.get(key).copied(),
    ///     Some("an example value")
    /// ));
    /// slotmap.remove(key);
    /// assert!(slotmap.get(key).is_none());
    /// ```
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

    /// Returns an exclusive reference to the value associated with the key and
    /// otherwise behaves indentically to `get`.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    /// let key = slotmap.insert("an example value".to_string());
    /// *slotmap.get_mut(key).unwrap() += " that has been mutated";
    /// assert!(matches!(
    ///     slotmap.get(key).map(|s| s.as_str()),
    ///     Some("an example value that has been mutated")
    /// ));
    /// ```
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

    /// Returns the number of occupied slots.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    ///
    /// for _ in 0..10 {
    ///     let _ = slotmap.insert(());
    /// }
    ///
    /// assert_eq!(slotmap.len(), 10);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if there are no occupied slots.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    /// let _ = slotmap.insert(());
    /// assert!(!slotmap.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_insert_get_remove() {
        let mut slotmap = SlotMap::new();
        let a = slotmap.insert("a");
        let b = slotmap.insert("b");
        let c = slotmap.insert("c");
        assert_eq!(*slotmap.get(a).unwrap(), "a");
        assert_eq!(*slotmap.get(b).unwrap(), "b");
        assert_eq!(*slotmap.get(c).unwrap(), "c");
        assert_eq!(*slotmap.get_mut(a).unwrap(), "a");
        assert_eq!(*slotmap.get_mut(b).unwrap(), "b");
        assert_eq!(*slotmap.get_mut(c).unwrap(), "c");
    }

    #[test]
    fn test_remove() {
        let mut slotmap = SlotMap::new();
        let a = slotmap.insert("a");
        let b = slotmap.insert("b");
        let c = slotmap.insert("c");
        assert_eq!(slotmap.remove(a).unwrap(), "a");
        assert_eq!(slotmap.remove(b).unwrap(), "b");
        assert_eq!(slotmap.remove(c).unwrap(), "c");
        assert_eq!(slotmap.get(a), None);
        assert_eq!(slotmap.get(b), None);
        assert_eq!(slotmap.get(c), None);
    }

    #[test]
    fn test_len() {
        let mut slotmap = SlotMap::new();
        assert!(slotmap.is_empty());
        let a = slotmap.insert(());
        let b = slotmap.insert(());
        let c = slotmap.insert(());
        assert_eq!(slotmap.len(), 3);
        slotmap.remove(a);
        slotmap.remove(b);
        slotmap.remove(c);
        assert!(slotmap.is_empty());
    }

    #[test]
    fn test_uaf() {
        let mut slotmap = SlotMap::new();
        let a = slotmap.insert("a");
        slotmap.remove(a);
        let b = slotmap.insert("b");
        assert_eq!(slotmap.get(a), None);
        assert_eq!(*slotmap.get(b).unwrap(), "b");
    }
}
