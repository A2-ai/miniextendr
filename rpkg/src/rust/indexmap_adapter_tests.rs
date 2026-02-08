//! IndexMap adapter tests
use miniextendr_api::indexmap_impl::IndexMap;
use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
pub fn indexmap_roundtrip_int(map: IndexMap<String, i32>) -> IndexMap<String, i32> {
    map
}

/// @noRd
#[miniextendr]
pub fn indexmap_roundtrip_str(map: IndexMap<String, String>) -> IndexMap<String, String> {
    map
}

/// @noRd
#[miniextendr]
pub fn indexmap_roundtrip_dbl(map: IndexMap<String, f64>) -> IndexMap<String, f64> {
    map
}

/// @noRd
#[miniextendr]
pub fn indexmap_keys(map: IndexMap<String, i32>) -> Vec<String> {
    map.keys().cloned().collect()
}

/// @noRd
#[miniextendr]
pub fn indexmap_len(map: IndexMap<String, i32>) -> i32 {
    map.len() as i32
}

/// Empty map roundtrip
/// @noRd
#[miniextendr]
pub fn indexmap_empty() -> IndexMap<String, i32> {
    IndexMap::new()
}

/// Duplicate key insert: later value wins in IndexMap
/// @noRd
#[miniextendr]
pub fn indexmap_duplicate_key() -> IndexMap<String, i32> {
    let mut map = IndexMap::new();
    map.insert("key".to_string(), 1);
    map.insert("key".to_string(), 2);
    map
}

/// Ordering preservation with many entries
/// @noRd
#[miniextendr]
pub fn indexmap_order_preserved() -> Vec<String> {
    let mut map = IndexMap::new();
    map.insert("z".to_string(), 1);
    map.insert("a".to_string(), 2);
    map.insert("m".to_string(), 3);
    map.insert("b".to_string(), 4);
    map.keys().cloned().collect()
}

/// Single entry map
/// @noRd
#[miniextendr]
pub fn indexmap_single() -> IndexMap<String, String> {
    let mut map = IndexMap::new();
    map.insert("only".to_string(), "value".to_string());
    map
}

miniextendr_module! {
    mod indexmap_adapter_tests;
    fn indexmap_roundtrip_int;
    fn indexmap_roundtrip_str;
    fn indexmap_roundtrip_dbl;
    fn indexmap_keys;
    fn indexmap_len;
    fn indexmap_empty;
    fn indexmap_duplicate_key;
    fn indexmap_order_preserved;
    fn indexmap_single;
}
