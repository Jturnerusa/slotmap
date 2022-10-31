#![deny(clippy::pedantic)]

mod macros;

pub mod standard;

type IterItem<'a, T> = (Key, &'a T);
type IterMutItem<'a, T> = (Key, &'a mut T);
type IntoIterItem<T> = (Key, T);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    index: usize,
    generation: u64,
}
