//! Tests for parameter default values via `#[miniextendr(default = "...")]`.

use miniextendr_api::{miniextendr, miniextendr_module};

/// @title Default Parameter Tests
/// @name rpkg_default_tests
/// @description Functions demonstrating default parameter support.
/// @examples
/// greet()
/// greet("Claude")
/// add_with_defaults(1L)
/// add_with_defaults(1L, 2L, 3L)
/// with_flag()
/// @aliases greet greet_hidden add_with_defaults with_flag
/// @param name Name to greet (default: "World").
/// @param x First integer to add.
/// @param y Second integer to add (default: 0).
/// @param z Third integer to add (default: 1).
/// @param flag Boolean flag (default: FALSE).
/// @return A greeting string or computed value.
///
/// Greets a person by name.
#[miniextendr]
pub fn greet(#[miniextendr(default = "\"World\"")] name: String) -> String {
    format!("Hello, {}!", name)
}

/// @rdname rpkg_default_tests
///
/// Internal greeting function (non-public).
/// @export
#[miniextendr]
fn greet_hidden(#[miniextendr(default = "\"World\"")] name: String) -> String {
    format!("Hello, {}!", name)
}

/// @rdname rpkg_default_tests
///
/// Adds three integers with defaults for y and z.
/// @export
#[miniextendr]
pub fn add_with_defaults(
    x: i32,
    #[miniextendr(default = "0L")] y: i32,
    #[miniextendr(default = "1L")] z: i32,
) -> i32 {
    x + y + z
}

/// @rdname rpkg_default_tests
///
/// Returns the flag value.
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
