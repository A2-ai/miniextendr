//! IndexMap adapter tests
use miniextendr_api::indexmap_impl::IndexMap;
use miniextendr_api::miniextendr;

/// Test IndexMap<String, i32> roundtrip through R named list.
/// @param map Named integer list from R.
#[miniextendr]
pub fn indexmap_roundtrip_int(map: IndexMap<String, i32>) -> IndexMap<String, i32> {
    map
}

/// Test IndexMap<String, String> roundtrip through R named list.
/// @param map Named character list from R.
#[miniextendr]
pub fn indexmap_roundtrip_str(map: IndexMap<String, String>) -> IndexMap<String, String> {
    map
}

/// Test IndexMap<String, f64> roundtrip through R named list.
/// @param map Named double list from R.
#[miniextendr]
pub fn indexmap_roundtrip_dbl(map: IndexMap<String, f64>) -> IndexMap<String, f64> {
    map
}

/// Test extracting keys from an IndexMap in insertion order.
/// @param map Named integer list from R.
#[miniextendr]
pub fn indexmap_keys(map: IndexMap<String, i32>) -> Vec<String> {
    map.keys().cloned().collect()
}

/// Test getting the number of entries in an IndexMap.
/// @param map Named integer list from R.
#[miniextendr]
pub fn indexmap_len(map: IndexMap<String, i32>) -> i32 {
    map.len() as i32
}

/// Test roundtripping an empty IndexMap.
#[miniextendr]
pub fn indexmap_empty() -> IndexMap<String, i32> {
    IndexMap::new()
}

/// Test that duplicate key insertion keeps the last value.
#[miniextendr]
pub fn indexmap_duplicate_key() -> IndexMap<String, i32> {
    let mut map = IndexMap::new();
    map.insert("key".to_string(), 1);
    map.insert("key".to_string(), 2);
    map
}

/// Test that insertion order is preserved in IndexMap keys.
#[miniextendr]
pub fn indexmap_order_preserved() -> Vec<String> {
    let mut map = IndexMap::new();
    map.insert("z".to_string(), 1);
    map.insert("a".to_string(), 2);
    map.insert("m".to_string(), 3);
    map.insert("b".to_string(), 4);
    map.keys().cloned().collect()
}

/// Test roundtripping a single-entry IndexMap.
#[miniextendr]
pub fn indexmap_single() -> IndexMap<String, String> {
    let mut map = IndexMap::new();
    map.insert("only".to_string(), "value".to_string());
    map
}

// region: RIndexMapOps adapter trait

/// Drive an `IndexMap` through the `RIndexMapOps` adapter trait (audit A7 —
/// the conversions above call inherent `IndexMap` methods; the trait was
/// unexercised). Calls are trait-qualified.
/// @param map Named integer list from R.
/// @param key Key to probe with `contains_key` / `get_index_of`.
#[miniextendr]
pub fn indexmap_ops_via_trait(map: IndexMap<String, i32>, key: &str) -> Vec<String> {
    use miniextendr_api::indexmap_impl::RIndexMapOps;

    vec![
        RIndexMapOps::len(&map).to_string(),
        RIndexMapOps::is_empty(&map).to_string(),
        RIndexMapOps::keys(&map).join(","),
        RIndexMapOps::contains_key(&map, key).to_string(),
        RIndexMapOps::get_index_of(&map, key).to_string(),
        RIndexMapOps::first(&map).map_or_else(String::new, |(k, v)| format!("{k}={v}")),
        RIndexMapOps::last(&map).map_or_else(String::new, |(k, v)| format!("{k}={v}")),
        RIndexMapOps::get_index(&map, 0).map_or_else(String::new, |(k, v)| format!("{k}={v}")),
        RIndexMapOps::get_key_at(&map, 1).unwrap_or_default(),
    ]
}

// endregion
