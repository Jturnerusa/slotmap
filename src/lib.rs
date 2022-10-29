pub mod standard;

type IterItem<'a, T> = (Key, &'a T);
type IterMutItem<'a, T> = (Key, &'a mut T);
type IntoIterItem<T> = (Key, T);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    index: usize,
    generation: u64,
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_insert_get_remove() {
//         let mut slotmap = SlotMap::new();
//         let a = slotmap.insert("a");
//         let b = slotmap.insert("b");
//         let c = slotmap.insert("c");
//         assert_eq!(*slotmap.get(a).unwrap(), "a");
//         assert_eq!(*slotmap.get(b).unwrap(), "b");
//         assert_eq!(*slotmap.get(c).unwrap(), "c");
//         assert_eq!(*slotmap.get_mut(a).unwrap(), "a");
//         assert_eq!(*slotmap.get_mut(b).unwrap(), "b");
//         assert_eq!(*slotmap.get_mut(c).unwrap(), "c");
//     }

//     #[test]
//     fn test_remove() {
//         let mut slotmap = SlotMap::new();
//         let a = slotmap.insert("a");
//         let b = slotmap.insert("b");
//         let c = slotmap.insert("c");
//         assert_eq!(slotmap.remove(a).unwrap(), "a");
//         assert_eq!(slotmap.remove(b).unwrap(), "b");
//         assert_eq!(slotmap.remove(c).unwrap(), "c");
//         assert_eq!(slotmap.get(a), None);
//         assert_eq!(slotmap.get(b), None);
//         assert_eq!(slotmap.get(c), None);
//     }

//     #[test]
//     fn test_len() {
//         let mut slotmap = SlotMap::new();
//         assert!(slotmap.is_empty());
//         let a = slotmap.insert(());
//         let b = slotmap.insert(());
//         let c = slotmap.insert(());
//         assert_eq!(slotmap.len(), 3);
//         slotmap.remove(a);
//         slotmap.remove(b);
//         slotmap.remove(c);
//         assert!(slotmap.is_empty());
//     }

//     #[test]
//     fn test_slot_reuse() {
//         let mut slotmap = SlotMap::new();
//         let a = slotmap.insert(());
//         let b = slotmap.insert(());
//         let c = slotmap.insert(());
//         slotmap.remove(a);
//         slotmap.remove(b);
//         slotmap.remove(c);
//         slotmap.insert(());
//         slotmap.insert(());
//         slotmap.insert(());
//         slotmap.insert(());
//         assert_eq!(slotmap.len(), 4);
//     }

//     #[test]
//     fn test_uaf() {
//         let mut slotmap = SlotMap::new();
//         let a = slotmap.insert("a");
//         slotmap.remove(a);
//         let b = slotmap.insert("b");
//         assert_eq!(slotmap.get(a), None);
//         assert_eq!(*slotmap.get(b).unwrap(), "b")
//     }

//     #[test]
//     fn test_iterator() {
//         let mut slotmap = SlotMap::new();
//         let a = slotmap.insert("a");
//         let b = slotmap.insert("b");
//         let c = slotmap.insert("c");
//         let mut iter = slotmap.iter();
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (b, "b"));
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
//         assert_eq!(iter.next(), None);
//         let mut iter = slotmap.iter_mut();
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (b, "b"));
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
//         assert_eq!(iter.next(), None);
//         let mut iter = slotmap.into_iter();
//         assert_eq!(iter.next().unwrap(), (a, "a"));
//         assert_eq!(iter.next().unwrap(), (b, "b"));
//         assert_eq!(iter.next().unwrap(), (c, "c"));
//         assert_eq!(iter.next(), None);
//     }

//     #[test]
//     fn test_iterator_skip_vacant() {
//         let mut slotmap = SlotMap::new();
//         let a = slotmap.insert("a");
//         let b = slotmap.insert("b");
//         let c = slotmap.insert("c");
//         slotmap.remove(b);
//         let mut iter = slotmap.iter();
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
//         assert_eq!(iter.next(), None);
//         let mut iter = slotmap.iter_mut();
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
//         assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
//         assert_eq!(iter.next(), None);
//         let mut iter = slotmap.into_iter();
//         assert_eq!(iter.next().unwrap(), (a, "a"));
//         assert_eq!(iter.next().unwrap(), (c, "c"));
//         assert_eq!(iter.next(), None);
//     }
// }
