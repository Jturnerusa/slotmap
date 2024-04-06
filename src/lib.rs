#!/bin/false

//! A map-like data structure that provides weak, light weight handles to inserted values.
//! ```
//! use slotmap::SlotMap;
//!
//! let mut slotmap = SlotMap::new();
//!
//! let a = slotmap.insert("hello");
//! let b = slotmap.insert("world");
//!
//! // Access values with the index operator. Indexing with invalid keys causes a panic.
//! assert_eq!(slotmap[a], "hello");
//!
//! // Use the get method if you aren't sure that a key is valid.
//! assert!(matches!(slotmap.get(b).cloned(), Some("world")));
//!
//! for (key, value) in &slotmap {
//!     println!("{:?} {}", key, value);
//! }
//!
//! // Removing values pops them out.
//! assert!(matches!(slotmap.remove(a), Some("hello")));
//! assert!(slotmap.get(a).is_none());
//!
//! // Double freeing values is safe.
//! assert!(slotmap.remove(a).is_none());
//! ```
//!
//! # Note
//! You should probably consider using the more widely used and battle tested
//! [slotmap crate](https://crates.io/crates/slotmap) rather than this one.

#![deny(clippy::pedantic)]

use std::ops::{Index, IndexMut};

pub struct Iter<'a, T: 'a>(std::slice::Iter<'a, Item<T>>);
pub struct IterMut<'a, T: 'a>(std::slice::IterMut<'a, Item<T>>);
pub struct IntoIter<T>(std::vec::IntoIter<Item<T>>);
pub struct Values<'a, T>(Iter<'a, T>);
pub struct ValuesMut<'a, T>(IterMut<'a, T>);
pub struct IntoValues<T>(IntoIter<T>);
pub struct Keys<'a, T>(Iter<'a, T>);

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

#[derive(Clone, Copy)]
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
/// Insertion, removal and access are constant time operations and involve
/// a single level of indirection to map the slot index to the item index.
///
/// Removing values doesn't require shifting any elements. This is done using
/// [`Vec::swap_remove`](Vec::swap_remove) interally and then updating the indirect indexes as needed.
/// Shrinking the underlying storage is not supported.
/// #### Iteration
/// All key value pairs are stored contigously in a vector, so iteration is as
/// fast as possible.
#[derive(Clone, Default)]
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
    /// a value still exists.
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

    /// Checks whether a key is still valid.
    ///
    /// This is functionally equivalent to calling `is_some`
    /// on the `Option` returned by `get`.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    /// let key = slotmap.insert("an example value");
    /// assert!(slotmap.contains_key(key));
    /// ```
    #[must_use]
    pub fn contains_key(&self, key: Key) -> bool {
        self.get(key).is_some()
    }

    /// Remove all items that do not satisfy a predicate.
    /// ##### Performance
    /// Removing elements does not require shifting elements but
    /// does change the ordering of items.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    ///
    /// for i in 0..10 {
    ///    let _ = slotmap.insert(i);
    /// }
    ///
    /// slotmap.retain(|(key, val)| val % 2 == 0);
    ///
    /// assert_eq!(slotmap.len(), 5);
    /// ```
    pub fn retain<F>(&mut self, f: F)
    where
        F: Fn((Key, &T)) -> bool,
    {
        let mut i = 0;
        while i < self.items.len() {
            let key = self.items[i].key;
            let val = &self.items[i].value;
            if f((key, val)) {
                i += 1;
            } else {
                self.remove(key);
            }
        }
    }

    /// Returns an iterator that yields a (key, value) tuple for every
    /// occupied slot in the slotmap.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    ///
    /// for i in 0..10 {
    ///    let _ = slotmap.insert(i * 2);
    /// }
    ///
    /// for (key, val) in &slotmap {
    ///    println!("{:?}: {}", key, val);
    /// }
    /// ```
    #[must_use]
    pub fn iter(&self) -> Iter<T> {
        Iter(self.items.iter())
    }

    /// See [`SlotMap::iter`](crate::SlotMap::iter)
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    ///
    /// for i in 0..10 {
    ///     let _ = slotmap.insert(i);
    /// }
    ///
    /// for (_key, val) in &mut slotmap {
    ///     *val *= 2;
    /// }
    /// ```
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut(self.items.iter_mut())
    }

    /// Iterate over values in the slotmap.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    ///
    /// for i in 0..10 {
    ///     let _ = slotmap.insert(i);
    /// }
    ///
    /// for value in slotmap.values() {
    ///     println!("{value}");
    /// }
    #[must_use]
    pub fn values(&self) -> Values<T> {
        Values(self.iter())
    }

    /// See [`SlotMap::values`](crate::SlotMap::values)
    ///
    /// Iterate over mutable references to values in the slotmap.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    ///
    /// for i in 0..10 {
    ///     let _ = slotmap.insert(i);
    /// }
    ///
    /// for mut value in slotmap.values_mut() {
    ///     *value += 1;
    /// }
    #[must_use]
    pub fn values_mut(&mut self) -> ValuesMut<T> {
        ValuesMut(self.iter_mut())
    }

    /// See [`SlotMap::values`](crate::SlotMap::values)
    ///
    /// Consume slotmap and iterate over the keys.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    ///
    /// for i in 0..10 {
    ///     let _ = slotmap.insert(i);
    /// }
    ///
    /// let values = slotmap.into_values().collect::<Vec<i32>>();
    /// ```
    #[must_use]
    pub fn into_values(self) -> IntoValues<T> {
        IntoValues(self.into_iter())
    }

    /// Iterate over keys in the slotmap.
    /// ##### Example
    /// ```
    /// use slotmap::SlotMap;
    ///
    /// let mut slotmap = SlotMap::new();
    ///
    /// for i in 0..10 {
    ///     let _ = slotmap.insert(i);
    /// }
    ///
    /// for key in slotmap.keys() {
    ///     println!("{:?}", key);
    /// }
    /// ```
    #[must_use]
    pub fn keys(&self) -> Keys<T> {
        Keys(self.iter())
    }
}

impl<T> Index<Key> for SlotMap<T> {
    type Output = T;
    fn index(&self, index: Key) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T> IndexMut<Key> for SlotMap<T> {
    fn index_mut(&mut self, index: Key) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Key, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|item| (item.key, &item.value)).next()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Key, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .by_ref()
            .map(|item| (item.key, &mut item.value))
            .next()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = (Key, T);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|item| (item.key, item.value)).next()
    }
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|(_, value)| value).next()
    }
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|(_, value)| value).next()
    }
}

impl<T> Iterator for IntoValues<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|(_, value)| value).next()
    }
}

impl<'a, T> Iterator for Keys<'a, T> {
    type Item = Key;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|(key, _)| key).next()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .by_ref()
            .map(|item| (item.key, &item.value))
            .next_back()
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .by_ref()
            .map(|item| (item.key, &mut item.value))
            .next_back()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .by_ref()
            .map(|item| (item.key, item.value))
            .next_back()
    }
}

impl<'a, T> DoubleEndedIterator for Values<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|(_, value)| value).next_back()
    }
}

impl<'a, T> DoubleEndedIterator for ValuesMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|(_, value)| value).next_back()
    }
}

impl<T> DoubleEndedIterator for IntoValues<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|(_, value)| value).next_back()
    }
}

impl<'a, T> DoubleEndedIterator for Keys<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.by_ref().map(|(key, _)| key).next_back()
    }
}

impl<'a, T> IntoIterator for &'a SlotMap<T> {
    type Item = (Key, &'a T);
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SlotMap<T> {
    type Item = (Key, &'a mut T);
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for SlotMap<T> {
    type Item = (Key, T);
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.items.into_iter())
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
        assert_eq!(a.index, b.index);
        assert_eq!(slotmap.get(a), None);
        assert_eq!(*slotmap.get(b).unwrap(), "b");
    }

    #[test]
    fn test_iter() {
        let mut slotmap = SlotMap::new();
        let mut keys = Vec::new();
        for x in 0..10 {
            keys.push(slotmap.insert(x));
        }
        let mut it = slotmap.iter();
        let a = it.by_ref().take(5).collect::<Vec<_>>();
        let b = it.by_ref().rev().collect::<Vec<_>>();
        assert!(matches!(
            a.as_slice(),
            [(_, 0), (_, 1), (_, 2), (_, 3), (_, 4)]
        ));
        assert!(matches!(
            dbg!(b.as_slice()),
            [(_, 9), (_, 8), (_, 7), (_, 6), (_, 5)]
        ));
    }

    #[test]
    fn test_iter_mut() {
        let mut slotmap = SlotMap::new();
        let mut keys = Vec::new();
        for x in 0..10 {
            keys.push(slotmap.insert(x));
        }
        for (_, val) in &mut slotmap {
            *val *= 2;
        }
        let mut it = slotmap.iter();
        let a = it.by_ref().take(5).collect::<Vec<_>>();
        let b = it.by_ref().rev().collect::<Vec<_>>();
        assert!(matches!(
            a.as_slice(),
            [(_, 0), (_, 2), (_, 4), (_, 6), (_, 8)]
        ));
        assert!(matches!(
            dbg!(b.as_slice()),
            [(_, 18), (_, 16), (_, 14), (_, 12), (_, 10)]
        ));
    }

    #[test]
    fn test_into_iter() {
        let mut slotmap = SlotMap::new();
        let mut keys = Vec::new();
        for x in 0..10 {
            keys.push(slotmap.insert(x));
        }
        let mut it = slotmap.into_iter();
        let a = it.by_ref().take(5).collect::<Vec<_>>();
        let b = it.by_ref().rev().collect::<Vec<_>>();
        assert!(matches!(
            a.as_slice(),
            [(_, 0), (_, 1), (_, 2), (_, 3), (_, 4)]
        ));
        assert!(matches!(
            dbg!(b.as_slice()),
            [(_, 9), (_, 8), (_, 7), (_, 6), (_, 5)]
        ));
    }

    #[test]
    fn test_values() {
        let mut slotmap = SlotMap::new();
        for x in 0..10 {
            let _ = slotmap.insert(x);
        }
        let mut it = slotmap.values();
        let a = it.by_ref().take(5).collect::<Vec<_>>();
        let b = it.by_ref().rev().collect::<Vec<_>>();
        assert!(matches!(a.as_slice(), [0, 1, 2, 3, 4]));
        assert!(matches!(b.as_slice(), [9, 8, 7, 6, 5]));
    }

    #[test]
    fn test_values_mut() {
        let mut slotmap = SlotMap::new();
        for x in 0..10 {
            let _ = slotmap.insert(x);
        }
        let mut it = slotmap.values_mut();
        let a = it.by_ref().take(5).collect::<Vec<_>>();
        let b = it.by_ref().rev().collect::<Vec<_>>();
        assert!(matches!(a.as_slice(), [0, 1, 2, 3, 4]));
        assert!(matches!(b.as_slice(), [9, 8, 7, 6, 5]));
    }

    #[test]
    fn test_into_values() {
        let mut slotmap = SlotMap::new();
        for x in 0..10 {
            let _ = slotmap.insert(x);
        }
        let mut it = slotmap.into_values();
        let a = it.by_ref().take(5).collect::<Vec<_>>();
        let b = it.by_ref().rev().collect::<Vec<_>>();
        assert!(matches!(a.as_slice(), [0, 1, 2, 3, 4]));
        assert!(matches!(b.as_slice(), [9, 8, 7, 6, 5]));
    }

    #[test]
    fn test_keys() {
        let mut slotmap = SlotMap::new();
        let mut keys = Vec::new();
        for x in 0..10 {
            keys.push(slotmap.insert(x));
        }
        let mut it = slotmap.keys();
        let a = it.by_ref().take(5).collect::<Vec<_>>();
        let b = it.by_ref().rev().take(5).collect::<Vec<_>>();
        assert_eq!(keys[0], a[0]);
        assert_eq!(keys[1], a[1]);
        assert_eq!(keys[2], a[2]);
        assert_eq!(keys[3], a[3]);
        assert_eq!(keys[4], a[4]);
        assert_eq!(keys[9], b[0]);
        assert_eq!(keys[8], b[1]);
        assert_eq!(keys[7], b[2]);
        assert_eq!(keys[6], b[3]);
        assert_eq!(keys[5], b[4]);
    }

    #[test]
    fn test_retain() {
        let mut slotmap = SlotMap::new();

        for x in 0..10 {
            let _ = slotmap.insert(x);
        }

        slotmap.retain(|(_, _)| true);
        assert_eq!(slotmap.len(), 10);

        slotmap.retain(|(_, val)| *val >= 5);
        assert_eq!(slotmap.len(), 5);

        slotmap.retain(|(_, _)| false);
        assert!(slotmap.is_empty());
    }
}
