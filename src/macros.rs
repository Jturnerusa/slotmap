#[doc(hidden)]
#[macro_export]
macro_rules! test_insert_get {
    ($type:ty) => {
        #[test]
        fn test_insert_get_remove() {
            let mut slotmap = <$type>::new();
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
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_remove {
    ($type:ty) => {
        #[test]
        fn test_remove() {
            let mut slotmap = <$type>::new();
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
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_len {
    ($type:ty) => {
        #[test]
        fn test_len() {
            let mut slotmap = <$type>::new();
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
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_uaf {
    ($type:ty) => {
        #[test]
        fn test_uaf() {
            let mut slotmap = <$type>::new();
            let a = slotmap.insert("a");
            slotmap.remove(a);
            let b = slotmap.insert("b");
            assert_eq!(slotmap.get(a), None);
            assert_eq!(*slotmap.get(b).unwrap(), "b")
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_iterator {
    ($type:ty) => {
        #[test]
        fn test_iterator() {
            let mut slotmap = <$type>::new();
            let a = slotmap.insert("a");
            let b = slotmap.insert("b");
            let c = slotmap.insert("c");
            let mut iter = slotmap.iter();
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (b, "b"));
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
            assert_eq!(iter.next(), None);
            let mut iter = slotmap.iter_mut();
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (b, "b"));
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
            assert_eq!(iter.next(), None);
            let mut iter = slotmap.into_iter();
            assert_eq!(iter.next().unwrap(), (a, "a"));
            assert_eq!(iter.next().unwrap(), (b, "b"));
            assert_eq!(iter.next().unwrap(), (c, "c"));
            assert_eq!(iter.next(), None);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_iterator_skip_vacant {
    ($type:ty) => {
        #[test]
        fn test_iterator_skip_vacant() {
            let mut slotmap = <$type>::new();
            let a = slotmap.insert("a");
            let b = slotmap.insert("b");
            let c = slotmap.insert("c");
            slotmap.remove(b);
            let mut iter = slotmap.iter();
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
            assert_eq!(iter.next(), None);
            let mut iter = slotmap.iter_mut();
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
            assert_eq!(iter.next().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
            assert_eq!(iter.next(), None);
            let mut iter = slotmap.into_iter();
            assert_eq!(iter.next().unwrap(), (a, "a"));
            assert_eq!(iter.next().unwrap(), (c, "c"));
            assert_eq!(iter.next(), None);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! test_double_ended_iterator {
    ($type:ty) => {
        #[test]
        fn test_double_ended_iterator() {
            let mut slotmap = <$type>::new();
            let a = slotmap.insert("a");
            let b = slotmap.insert("b");
            let c = slotmap.insert("c");
            let mut iter = slotmap.iter();
            assert_eq!(iter.next_back().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
            assert_eq!(iter.next_back().map(|(k, v)| (k, *v)).unwrap(), (b, "b"));
            assert_eq!(iter.next_back().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
            assert_eq!(iter.next_back(), None);
            let mut iter = slotmap.iter_mut();
            assert_eq!(iter.next_back().map(|(k, v)| (k, *v)).unwrap(), (c, "c"));
            assert_eq!(iter.next_back().map(|(k, v)| (k, *v)).unwrap(), (b, "b"));
            assert_eq!(iter.next_back().map(|(k, v)| (k, *v)).unwrap(), (a, "a"));
            assert_eq!(iter.next_back(), None);
            let mut iter = slotmap.into_iter();
            assert_eq!(iter.next_back().unwrap(), (c, "c"));
            assert_eq!(iter.next_back().unwrap(), (b, "b"));
            assert_eq!(iter.next_back().unwrap(), (a, "a"));
            assert_eq!(iter.next_back(), None);
        }
    };
}

pub use test_double_ended_iterator;
pub use test_insert_get;
pub use test_iterator;
pub use test_iterator_skip_vacant;
pub use test_len;
pub use test_remove;
pub use test_uaf;
