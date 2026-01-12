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

miniextendr_module! {
    mod indexmap_adapter_tests;
    fn indexmap_roundtrip_int;
    fn indexmap_roundtrip_str;
    fn indexmap_roundtrip_dbl;
    fn indexmap_keys;
    fn indexmap_len;
}
