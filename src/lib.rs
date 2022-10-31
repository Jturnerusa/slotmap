#![deny(clippy::pedantic)]

mod macros;

pub mod standard;

type IterItem<'a, T> = (Key, &'a T);
type IterMutItem<'a, T> = (Key, &'a mut T);
type IntoIterItem<T> = (Key, T);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Generation(pub u64);

impl Generation {
    pub fn next(self) -> Generation {
        Generation(self.0 + 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    index: usize,
    generation: Generation,
}
