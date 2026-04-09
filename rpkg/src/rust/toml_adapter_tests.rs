//! TOML adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::toml_impl::{
    RTomlOps, TomlValue, toml_from_str, toml_to_string, toml_to_string_pretty,
};

/// Test TOML string roundtrip via parsing and re-serialization.
/// @param input TOML string to parse and re-serialize.
#[miniextendr]
pub fn toml_roundtrip(input: String) -> String {
    let v = toml_from_str(&input).unwrap();
    toml_to_string(&v)
}

/// Test pretty-printing a TOML string.
/// @param input TOML string to parse and pretty-print.
#[miniextendr]
pub fn toml_pretty(input: String) -> String {
    let v = toml_from_str(&input).unwrap();
    toml_to_string_pretty(&v)
}

/// Test getting the TOML type name of a value.
/// @param input TOML value from R.
#[miniextendr]
pub fn toml_type_name(input: TomlValue) -> String {
    input.type_name()
}

/// Test whether a TOML value is a table.
/// @param input TOML value from R.
#[miniextendr]
pub fn toml_is_table(input: TomlValue) -> bool {
    input.is_table()
}

/// Test extracting top-level keys from a TOML table.
/// @param input TOML table value from R.
#[miniextendr]
pub fn toml_table_keys(input: TomlValue) -> Vec<String> {
    input.table_keys()
}

/// Test extracting keys from nested TOML tables.
/// @param input TOML string with nested tables.
#[miniextendr]
pub fn toml_nested_keys(input: String) -> Vec<String> {
    let v = toml_from_str(&input).unwrap();
    v.as_table()
        .map(|t| t.keys().cloned().collect())
        .unwrap_or_default()
}

/// Test parsing and re-serializing TOML array of tables.
#[miniextendr]
pub fn toml_array_of_tables() -> String {
    let input = r#"
[[items]]
name = "a"

[[items]]
name = "b"
"#;
    let v = toml_from_str(input).unwrap();
    toml_to_string(&v)
}

/// Test that invalid TOML input returns an error message.
/// @param input Invalid TOML string.
#[miniextendr]
pub fn toml_parse_invalid(input: String) -> String {
    match toml_from_str(&input) {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

/// Test parsing TOML with mixed types (boolean, integer, string).
#[miniextendr]
pub fn toml_mixed_types() -> String {
    let input = "flag = true\ncount = 42\nname = \"test\"";
    let v = toml_from_str(input).unwrap();
    toml_to_string_pretty(&v)
}

// region: Upstream example-derived fixtures

/// Decode a TOML string and extract specific fields as a character vector.
/// Returns [title, version, author] from a typical config TOML.
/// @param input TOML string with title, version, and author fields.
#[miniextendr]
pub fn toml_decode_config(input: String) -> Vec<String> {
    let v = toml_from_str(&input).unwrap();
    let table = v.as_table().unwrap();
    vec![
        table
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        table
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        table
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    ]
}

/// Extract a string value from a TOML table by key.
/// @param input TOML string to parse.
/// @param key Key to extract.
#[miniextendr]
pub fn toml_get_string(input: String, key: &str) -> Option<String> {
    let v = toml_from_str(&input).unwrap();
    v.as_table()
        .and_then(|t| t.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Test counting entries in a TOML array of tables.
/// @param input TOML string containing an array of tables.
/// @param key Key name of the array of tables.
#[miniextendr]
pub fn toml_array_count(input: String, key: &str) -> i32 {
    let v = toml_from_str(&input).unwrap();
    v.as_table()
        .and_then(|t| t.get(key))
        .and_then(|v| v.as_array())
        .map(|a| a.len() as i32)
        .unwrap_or(0)
}

// endregion
