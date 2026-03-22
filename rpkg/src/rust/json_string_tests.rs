//! Test fixtures for AsJson, AsJsonPretty, FromJson, AsJsonVec wrappers.

use crate::serde::{Deserialize, Serialize};
use miniextendr_api::miniextendr;
use miniextendr_api::serde::{AsJson, AsJsonPretty, AsJsonVec, FromJson};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "crate::serde")]
pub struct Point {
    x: f64,
    y: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "crate::serde")]
pub struct Config {
    max_threads: i32,
    name: String,
}

// region: AsJson — compact JSON output

/// @export
#[miniextendr]
pub fn test_json_point() -> AsJson<Point> {
    AsJson(Point { x: 1.5, y: 2.5 })
}

/// @export
#[miniextendr]
pub fn test_json_config() -> AsJson<Config> {
    AsJson(Config {
        max_threads: 4,
        name: "test".into(),
    })
}

// endregion

// region: AsJsonPretty — pretty-printed JSON

/// @export
#[miniextendr]
pub fn test_json_pretty_point() -> AsJsonPretty<Point> {
    AsJsonPretty(Point { x: 1.0, y: 2.0 })
}

// endregion

// region: FromJson — parse JSON from R character

/// @export
#[miniextendr]
pub fn test_fromjson_config(json: FromJson<Config>) -> i32 {
    json.0.max_threads
}

/// @export
#[miniextendr]
pub fn test_fromjson_point_sum(json: FromJson<Point>) -> f64 {
    json.0.x + json.0.y
}

/// @export
#[miniextendr]
pub fn test_fromjson_bad(json: FromJson<Config>) -> i32 {
    json.0.max_threads
}

// endregion

// region: AsJsonVec — Vec<T> → character vector of JSON strings

/// @export
#[miniextendr]
pub fn test_json_vec_points() -> AsJsonVec<Point> {
    AsJsonVec(vec![
        Point { x: 1.0, y: 2.0 },
        Point { x: 3.0, y: 4.0 },
    ])
}

// endregion
