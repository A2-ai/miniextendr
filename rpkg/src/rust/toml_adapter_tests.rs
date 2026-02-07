//! TOML adapter tests
use miniextendr_api::toml_impl::{toml_from_str, toml_to_string, toml_to_string_pretty, RTomlOps, TomlValue};
use miniextendr_api::{miniextendr, miniextendr_module};

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

miniextendr_module! {
    mod toml_adapter_tests;
    fn toml_roundtrip;
    fn toml_pretty;
    fn toml_type_name;
    fn toml_is_table;
    fn toml_table_keys;
}
