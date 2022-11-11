//! A map-like data structure that provides weak, light weight handles to inserted values.
//! # Example use
//! ```
//! use slotmap::StandardSlotMap as SlotMap;
//!
//! let mut slotmap = SlotMap::new();
//!
//! // inserting a value returns a key
//! let key = slotmap.insert("hello".to_string());
//!
//! // keys are lightweight and impl Copy
//! let array_of_keys = [key; 10];
//!
//! // we can use keys to access values
//! slotmap[key] += " world!";
//! assert_eq!(slotmap[key].as_str(), "hello world!");
//!
//! // removing values moves them back to the caller
//! // in this case we just ignore the removed string
//! let _ = slotmap.remove(key);
//!
//! // trying to use stale keys is harmless
//! assert!(matches!(slotmap.get(key), None));
//!
//! // iterating the keys and values is supported
//! for (k, v) in &slotmap {
//!     println!("{:?} {}", k, v);
//! }
//! ```
//! # Note
//! You should probably consider using the more widely used and battle tested
//! [slotmap crate](https://crates.io/crates/slotmap) rather than this one.

#![deny(clippy::pedantic)]

mod macros;

#[allow(dead_code, unused_variables)]
pub mod indirection;

pub mod standard;

#[doc(inline)]
pub use standard::SlotMap as StandardSlotMap;

#[doc(inline)]
pub use indirection::SlotMap as IndirectionSlotMap;

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
/// The key is the size of a `u64` + `usize`, which is 16 bytes on 64 bit platforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    index: usize,
    generation: Generation,
}
