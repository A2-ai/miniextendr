//! Tests for parameter default values via `#[miniextendr(default = "...")]`.

use miniextendr_api::{SEXP, miniextendr};

/// Test greeting with a default name parameter.
/// @param name Character name to greet (defaults to "World").
#[miniextendr]
pub fn greet(#[miniextendr(default = "\"World\"")] name: String) -> String {
    format!("Hello, {}!", name)
}

/// Test greeting with a hidden default parameter.
/// @param name Character name to greet (defaults to "World").
#[miniextendr(noexport)]
pub fn greet_hidden(#[miniextendr(default = "\"World\"")] name: String) -> String {
    format!("Hello, {}!", name)
}

/// Test addition with multiple default parameters.
/// @param x Integer first operand.
/// @param y Integer second operand (defaults to 0L).
/// @param z Integer third operand (defaults to 1L).
#[miniextendr]
pub fn add_with_defaults(
    x: i32,
    #[miniextendr(default = "0L")] y: i32,
    #[miniextendr(default = "1L")] z: i32,
) -> i32 {
    x + y + z
}

/// Test boolean parameter with a default value.
/// @param flag Logical flag (defaults to FALSE).
#[miniextendr]
pub fn with_flag(#[miniextendr(default = "FALSE")] flag: bool) -> bool {
    flag
}

/// Test an optional `SEXP` parameter that defaults to `NULL`.
///
/// Returns `TRUE` when `value` is `NULL`. Omitting `value` and passing `NULL`
/// both reach Rust as R's `NULL`, which `SEXP::is_nil()` recognises. R never
/// passes a C null pointer, so a null-pointer test would be `FALSE` for every
/// call.
/// @param value Any R value (defaults to NULL).
#[miniextendr]
pub fn sexp_arg_is_nil(#[miniextendr(default = "NULL")] value: SEXP) -> bool {
    value.is_nil()
}
