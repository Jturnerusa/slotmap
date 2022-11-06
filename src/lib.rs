//! A map-like data structure that provides weak, light weight handles to inserted values.
//! # Example use
//! ```
//! use slotmap::StandardSlotMap;
//!
//! let mut slotmap = StandardSlotMap::new();
//!
//! // insert a value
//! let a = slotmap.insert("a".to_owned());
//!
//! // access the values via their associated keys
//! assert_eq!(slotmap[a], "a");
//!
//! // mutate a value
//! slotmap[a] = "new a".to_owned();
//! assert_eq!(slotmap[a], "new a");
//!
//! // remove a value
//! let removed = slotmap.remove(a).unwrap();
//! assert_eq!(removed.as_str(), "new a");
//!
//! // using a stale key doesn't work
//! assert!(matches!(slotmap.get(a), None));
//! ```

#![deny(clippy::pedantic)]

#[allow(dead_code, unused_variables)]
pub mod indirection;
mod macros;
pub mod standard;

#[doc(inline)]
pub use standard::SlotMap as StandardSlotMap;

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

/// A unique handle to a value in a slotmap.
///
/// The key stores a generation that gets compared to the generation stored in a
/// specific slot inside of the slotmap. When these generations match we know that
/// your key is not stale. Without this check, you would not be able to detect if
/// a key has already had it's value removed, which would cause the slotmap to
/// return incorrect data in use after free situations.
///
/// # Memory use
/// The key is the size of a [u64](u64) + [usize](usize), which is 16 bytes on 64 bit platforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    index: usize,
    generation: Generation,
}
