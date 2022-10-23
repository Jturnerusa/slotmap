use std::mem;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    index: usize,
    generation: u64,
}

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

pub struct Iter<'a, T>(::std::iter::Enumerate<::std::slice::Iter<'a, Slot<T>>>);

pub struct IterMut<'a, T>(::std::iter::Enumerate<::std::slice::IterMut<'a, Slot<T>>>);

pub struct IntoIter<T>(::std::iter::Enumerate<::std::vec::IntoIter<Slot<T>>>);

impl<T> SlotMap<T> {
    pub fn new() -> SlotMap<T> {
        SlotMap {
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
            Some(Slot::Occupied(g, item)) if *g == key.generation => Some(item),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        match self.slots.get_mut(key.index) {
            Some(Slot::Occupied(g, value)) if *g == key.generation => Some(value),
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
    type Item = (&'a T, Key);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut SlotMap<T> {
    type Item = (&'a mut T, Key);
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for SlotMap<T> {
    type Item = (T, Key);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.slots.into_iter().enumerate())
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a T, Key);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.find_map(|(index, slot)| match slot {
            Slot::Occupied(generation, value) => Some((
                value,
                Key {
                    index,
                    generation: *generation,
                },
            )),
            _ => None,
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (&'a mut T, Key);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.find_map(|(index, slot)| match slot {
            Slot::Occupied(generation, value) => Some((
                value,
                Key {
                    index,
                    generation: *generation,
                },
            )),
            _ => None,
        })
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = (T, Key);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.find_map(|(index, slot)| match slot {
            Slot::Occupied(generation, value) => Some((value, Key { index, generation })),
            _ => None,
        })
    }
}
