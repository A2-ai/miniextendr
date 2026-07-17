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

// region: Upstream example-derived fixtures

/// Test whether a JSON value is an array.
/// @param value JSON value from R.
#[miniextendr]
pub fn json_is_array(value: JsonValue) -> bool {
    value.is_array()
}

/// Test whether a JSON value is a string.
/// @param value JSON value from R.
#[miniextendr]
pub fn json_is_string(value: JsonValue) -> bool {
    value.is_string()
}

/// Test whether a JSON value is a number.
/// @param value JSON value from R.
#[miniextendr]
pub fn json_is_number(value: JsonValue) -> bool {
    value.is_number()
}

/// Test whether a JSON value is null.
/// @param value JSON value from R.
#[miniextendr]
pub fn json_is_null(value: JsonValue) -> bool {
    value.is_null()
}

/// Get the length of a JSON array, or 0 if not an array.
/// @param value JSON value from R.
#[miniextendr]
pub fn json_array_len(value: JsonValue) -> i32 {
    value.as_array().map(|a| a.len() as i32).unwrap_or(0)
}

/// Serialize a map from key-value pairs to JSON string.
/// @param keys Character vector of keys.
/// @param values Integer vector of values (one per key).
#[miniextendr]
pub fn json_from_key_values(keys: Vec<String>, values: Vec<i32>) -> String {
    use miniextendr_api::serde_impl::serde_json;
    let mut map = serde_json::Map::new();
    for (k, v) in keys.into_iter().zip(values) {
        map.insert(k, serde_json::Value::Number(v.into()));
    }
    serde_json::Value::Object(map).to_string()
}

// endregion

// region: RDeserialize trait + JsonOptions (audit A7)

/// Parse JSON through the `RDeserialize` adapter trait's `from_json`
/// (`None` → `NULL`-ish sentinel here) — the fixtures above only exercise the
/// `RSerialize` side.
/// @param json JSON string encoding a Point.
#[miniextendr]
pub fn json_rdeserialize_from_json(json: &str) -> f64 {
    use miniextendr_api::serde_impl::RDeserialize;

    match <Point as RDeserialize>::from_json(json) {
        Some(p) => p.x + p.y,
        None => f64::NAN,
    }
}

/// Parse JSON through `RDeserialize::from_json_result`, surfacing the parse
/// error text on failure.
/// @param json JSON string encoding a Point.
#[miniextendr]
pub fn json_rdeserialize_from_json_result(json: &str) -> String {
    use miniextendr_api::serde_impl::RDeserialize;

    match <Point as RDeserialize>::from_json_result(json) {
        Ok(p) => format!("ok:{}", p.x + p.y),
        Err(e) => format!("err:{e}"),
    }
}

/// Serialize an R object to JSON under `JsonOptions::permissive()` —
/// NA / NaN / Inf all become `null` instead of erroring.
/// @param value R object to serialize.
#[miniextendr]
pub fn json_options_permissive(value: miniextendr_api::prelude::SEXP) -> Result<String, String> {
    use miniextendr_api::serde_impl::{JsonOptions, json_from_sexp_with};

    json_from_sexp_with(value, &JsonOptions::permissive())
        .map(|v| v.to_string())
        .map_err(|e| e.to_string())
}

/// Serialize an R object to JSON under `JsonOptions::strict()` — special
/// values produce an `Err` (surfaced to R as a condition when unwrapped).
/// @param value R object to serialize.
#[miniextendr]
pub fn json_options_strict(value: miniextendr_api::prelude::SEXP) -> Result<String, String> {
    use miniextendr_api::serde_impl::{JsonOptions, json_from_sexp_with};

    json_from_sexp_with(value, &JsonOptions::strict())
        .map(|v| v.to_string())
        .map_err(|e| e.to_string())
}

/// Exercise the `JsonOptions` builder methods (`new` + `na`/`nan`/`inf`):
/// NA serializes as the string `"NA"`, NaN/Inf as `null`.
/// @param value R object to serialize.
#[miniextendr]
pub fn json_options_builder(value: miniextendr_api::prelude::SEXP) -> Result<String, String> {
    use miniextendr_api::serde_impl::{
        JsonOptions, NaHandling, SpecialFloatHandling, json_from_sexp_with,
    };

    let opts = JsonOptions::new()
        .na(NaHandling::String("NA".into()))
        .nan(SpecialFloatHandling::Null)
        .inf(SpecialFloatHandling::Null);
    json_from_sexp_with(value, &opts)
        .map(|v| v.to_string())
        .map_err(|e| e.to_string())
}

// endregion
