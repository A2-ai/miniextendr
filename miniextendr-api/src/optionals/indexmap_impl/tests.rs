use super::*;

/// `no_na` reads each top-level value by its own type, like the
/// `HashMap` / `BTreeMap` impls.
#[test]
fn indexmap_forwards_no_na_to_its_values() {
    use crate::convert::AsNumeric;

    let mut map = IndexMap::new();
    map.insert("a".to_string(), AsNumeric(Some(1.0)));
    assert!(!map.__mx_has_na());
    map.insert("b".to_string(), AsNumeric(None));
    assert!(map.__mx_has_na());
}

#[test]
fn indexmap_preserves_order() {
    let mut map = IndexMap::new();
    map.insert("z".to_string(), 1);
    map.insert("a".to_string(), 2);
    map.insert("m".to_string(), 3);

    let keys: Vec<_> = map.keys().cloned().collect();
    assert_eq!(keys, vec!["z", "a", "m"]);
}

#[test]
fn indexmap_capacity() {
    let map: IndexMap<String, i32> = IndexMap::with_capacity(10);
    assert!(map.capacity() >= 10);
}

#[test]
fn rindexmapops_len_and_empty() {
    let empty: IndexMap<String, i32> = IndexMap::new();
    assert_eq!(RIndexMapOps::len(&empty), 0);
    assert!(RIndexMapOps::is_empty(&empty));

    let mut map = IndexMap::new();
    map.insert("a".to_string(), 1);
    assert_eq!(RIndexMapOps::len(&map), 1);
    assert!(!RIndexMapOps::is_empty(&map));
}

#[test]
fn rindexmapops_keys() {
    let mut map = IndexMap::new();
    map.insert("z".to_string(), 1);
    map.insert("a".to_string(), 2);
    map.insert("m".to_string(), 3);

    let keys = RIndexMapOps::keys(&map);
    assert_eq!(keys, vec!["z", "a", "m"]);
}

#[test]
fn rindexmapops_contains_key() {
    let mut map = IndexMap::new();
    map.insert("foo".to_string(), 42);

    assert!(RIndexMapOps::contains_key(&map, "foo"));
    assert!(!RIndexMapOps::contains_key(&map, "bar"));
}

#[test]
fn rindexmapops_get_index() {
    let mut map = IndexMap::new();
    map.insert("first".to_string(), 1);
    map.insert("second".to_string(), 2);

    let (k, v) = RIndexMapOps::get_index(&map, 0).unwrap();
    assert_eq!(k, "first");
    assert_eq!(v, 1);

    let (k, v) = RIndexMapOps::get_index(&map, 1).unwrap();
    assert_eq!(k, "second");
    assert_eq!(v, 2);

    assert!(RIndexMapOps::get_index(&map, 2).is_none());
    assert!(RIndexMapOps::get_index(&map, -1).is_none());
}

#[test]
fn rindexmapops_get_key_at() {
    let mut map = IndexMap::new();
    map.insert("first".to_string(), 1);
    map.insert("second".to_string(), 2);

    assert_eq!(RIndexMapOps::get_key_at(&map, 0), Some("first".to_string()));
    assert_eq!(
        RIndexMapOps::get_key_at(&map, 1),
        Some("second".to_string())
    );
    assert_eq!(RIndexMapOps::get_key_at(&map, 2), None);
}

#[test]
fn rindexmapops_first_last() {
    let mut map = IndexMap::new();
    map.insert("a".to_string(), 1);
    map.insert("b".to_string(), 2);
    map.insert("c".to_string(), 3);

    let (k, v) = RIndexMapOps::first(&map).unwrap();
    assert_eq!(k, "a");
    assert_eq!(v, 1);

    let (k, v) = RIndexMapOps::last(&map).unwrap();
    assert_eq!(k, "c");
    assert_eq!(v, 3);

    let empty: IndexMap<String, i32> = IndexMap::new();
    assert!(RIndexMapOps::<i32>::first(&empty).is_none());
    assert!(RIndexMapOps::<i32>::last(&empty).is_none());
}

#[test]
fn rindexmapops_get_index_of() {
    let mut map = IndexMap::new();
    map.insert("a".to_string(), 1);
    map.insert("b".to_string(), 2);
    map.insert("c".to_string(), 3);

    assert_eq!(RIndexMapOps::get_index_of(&map, "a"), 0);
    assert_eq!(RIndexMapOps::get_index_of(&map, "b"), 1);
    assert_eq!(RIndexMapOps::get_index_of(&map, "c"), 2);
    assert_eq!(RIndexMapOps::get_index_of(&map, "d"), -1);
}
