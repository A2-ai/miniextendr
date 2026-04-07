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

/// Test JSON value roundtrip through R.
/// @param value JSON value from R (list, vector, or scalar).
#[miniextendr]
pub fn json_roundtrip(value: JsonValue) -> JsonValue {
    value
}

/// Test getting the JSON type name of a value.
/// @param value JSON value from R.
#[miniextendr]
pub fn json_type_name(value: JsonValue) -> String {
    value.type_name()
}

/// Test whether a JSON value is an object.
/// @param value JSON value from R.
#[miniextendr]
pub fn json_is_object(value: JsonValue) -> bool {
    value.is_object()
}

/// Test extracting keys from a JSON object.
/// @param value JSON object from R.
#[miniextendr]
pub fn json_object_keys(value: JsonValue) -> Vec<String> {
    value.object_keys()
}

/// Test serializing a Point struct to JSON string.
/// @param x X coordinate.
/// @param y Y coordinate.
#[miniextendr]
pub fn json_serialize_point(x: f64, y: f64) -> String {
    let p = Point { x, y };
    p.to_json().unwrap()
}

/// Test pretty-printing a JSON value.
/// @param value JSON value from R.
#[miniextendr]
pub fn json_to_pretty(value: JsonValue) -> String {
    value.to_json_string_pretty()
}
