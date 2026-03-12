//! serde_json adapter tests
use crate::serde::{Deserialize, Serialize};
use miniextendr_api::miniextendr;
use miniextendr_api::serde_impl::{JsonValue, RJsonValueOps, RSerialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "crate::serde")]
struct Point {
    x: f64,
    y: f64,
}

/// @noRd
#[miniextendr]
pub fn json_roundtrip(value: JsonValue) -> JsonValue {
    value
}

/// @noRd
#[miniextendr]
pub fn json_type_name(value: JsonValue) -> String {
    value.type_name()
}

/// @noRd
#[miniextendr]
pub fn json_is_object(value: JsonValue) -> bool {
    value.is_object()
}

/// @noRd
#[miniextendr]
pub fn json_object_keys(value: JsonValue) -> Vec<String> {
    value.object_keys()
}

/// @noRd
#[miniextendr]
pub fn json_serialize_point(x: f64, y: f64) -> String {
    let p = Point { x, y };
    p.to_json().unwrap()
}

/// @noRd
#[miniextendr]
pub fn json_to_pretty(value: JsonValue) -> String {
    value.to_json_string_pretty()
}
