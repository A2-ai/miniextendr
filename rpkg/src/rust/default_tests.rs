//! Tests for parameter default values via `#[miniextendr(default = "...")]`.

use miniextendr_api::miniextendr;

/// @noRd
#[miniextendr]
pub fn greet(#[miniextendr(default = "\"World\"")] name: String) -> String {
    format!("Hello, {}!", name)
}

/// @noRd
#[miniextendr]
fn greet_hidden(#[miniextendr(default = "\"World\"")] name: String) -> String {
    format!("Hello, {}!", name)
}

/// @noRd
#[miniextendr]
pub fn add_with_defaults(
    x: i32,
    #[miniextendr(default = "0L")] y: i32,
    #[miniextendr(default = "1L")] z: i32,
) -> i32 {
    x + y + z
}

/// @noRd
#[miniextendr]
pub fn with_flag(#[miniextendr(default = "FALSE")] flag: bool) -> bool {
    flag
}
