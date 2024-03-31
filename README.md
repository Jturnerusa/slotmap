 A map-like data structure that provides weak, light weight handles to inserted values.
 ```
 use slotmap::SlotMap;

 let mut slotmap = SlotMap::new();

 let a = slotmap.insert("hello");
 let b = slotmap.insert("world");

 // Access values with the index operator. Indexing with invalid keys causes a panic.
 assert_eq!(slotmap[a], "hello");

 // Use the get method if you aren't sure that a key is valid.
 assert!(matches!(slotmap.get(b).cloned(), Some("world")));

 for (key, value) in &slotmap {
     println!("{:?} {}", key, value);
 }

 // Removing values pops them out.
 assert!(matches!(slotmap.remove(a), Some("hello")));
 assert!(slotmap.get(a).is_none());

 // Double freeing values is safe.
 assert!(slotmap.remove(a).is_none());
 ```

 # Note
 You should probably consider using the more widely used and battle tested
 [slotmap crate](https://crates.io/crates/slotmap) rather than this one.
