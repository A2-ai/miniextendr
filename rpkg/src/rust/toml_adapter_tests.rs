//! TOML adapter tests
use miniextendr_api::miniextendr;
use miniextendr_api::toml_impl::{
    RTomlOps, TomlValue, toml_from_str, toml_to_string, toml_to_string_pretty,
};

/// @noRd
#[miniextendr]
pub fn toml_roundtrip(input: String) -> String {
    let v = toml_from_str(&input).unwrap();
    toml_to_string(&v)
}

/// @noRd
#[miniextendr]
pub fn toml_pretty(input: String) -> String {
    let v = toml_from_str(&input).unwrap();
    toml_to_string_pretty(&v)
}

/// @noRd
#[miniextendr]
pub fn toml_type_name(input: TomlValue) -> String {
    input.type_name()
}

/// @noRd
#[miniextendr]
pub fn toml_is_table(input: TomlValue) -> bool {
    input.is_table()
}

/// @noRd
#[miniextendr]
pub fn toml_table_keys(input: TomlValue) -> Vec<String> {
    input.table_keys()
}

/// Nested tables
/// @noRd
#[miniextendr]
pub fn toml_nested_keys(input: String) -> Vec<String> {
    let v = toml_from_str(&input).unwrap();
    v.as_table()
        .map(|t| t.keys().cloned().collect())
        .unwrap_or_default()
}

/// Array of tables (e.g. [[items]])
/// @noRd
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

/// Invalid TOML returns error message
/// @noRd
#[miniextendr]
pub fn toml_parse_invalid(input: String) -> String {
    match toml_from_str(&input) {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

/// Boolean and integer values in TOML
/// @noRd
#[miniextendr]
pub fn toml_mixed_types() -> String {
    let input = "flag = true\ncount = 42\nname = \"test\"";
    let v = toml_from_str(input).unwrap();
    toml_to_string_pretty(&v)
}
