//! Tests for parameter default values via `#[miniextendr(default = "...")]`.

use miniextendr_api::{miniextendr, miniextendr_module};

/// Test function with default parameter.
#[miniextendr]
pub fn greet(#[miniextendr(default = "\"World\"")] name: String) -> String {
    format!("Hello, {}!", name)
}

/// Test function with default parameter.
/// 
/// @export
#[miniextendr]
fn greet_hidden(#[miniextendr(default = "\"World\"")] name: String) -> String {
    format!("Hello, {}!", name)
}

/// Test function with multiple defaults.
/// @export
#[miniextendr]
pub fn add_with_defaults(
    x: i32,
    #[miniextendr(default = "0L")] y: i32,
    #[miniextendr(default = "1L")] z: i32,
) -> i32 {
    x + y + z
}

/// Test function with logical default.
/// @export
#[miniextendr]
pub fn with_flag(#[miniextendr(default = "FALSE")] flag: bool) -> bool {
    flag
}

miniextendr_module! {
    mod default_tests;
    fn greet;
    fn greet_hidden;
    fn add_with_defaults;
    fn with_flag;
}
