//! Tests for R dots (`...`) handling.

use miniextendr_api::{miniextendr, miniextendr_module, typed_list};

#[miniextendr]
/// @title Dots Handling Tests
/// @name rpkg_dots
/// @description Dots (`...`) handling tests
/// @examples
/// \dontrun{
/// greetings_with_named_dots(a = 1, b = 2)
/// greetings_with_nameless_dots(1, 2, 3)
/// greetings_last_as_named_dots(1L, x = 1, y = 2)
/// }
/// @aliases greetings_with_named_dots greetings_with_named_and_unused_dots
///   greetings_with_nameless_dots greetings_last_as_named_dots
///   greetings_last_as_named_and_unused_dots greetings_last_as_nameless_dots
/// @param dots Additional arguments (captured as dots).
pub fn greetings_with_named_dots(dots: ...) {
    let _ = dots;
}

#[miniextendr]
pub fn greetings_with_named_and_unused_dots(_dots: ...) {}

#[miniextendr]
pub fn greetings_with_nameless_dots(...) {}

// LIMITATION: Good!
// #[miniextendr]
// fn greetings_with_dots_then_arg(dots: ..., exclamations: i32) {}

#[miniextendr]
pub fn greetings_last_as_named_and_unused_dots(_exclamations: i32, _dots: ...) {}

#[miniextendr]
pub fn greetings_last_as_named_dots(_exclamations: i32, dots: ...) {
    let _ = dots;
}

#[miniextendr]
pub fn greetings_last_as_nameless_dots(_exclamations: i32, ...) {}

// =============================================================================
// typed_list! macro examples
// =============================================================================

/// Validate dots with typed_list! macro.
///
/// # Example from R
/// ```r
/// validate_numeric_args(alpha = c(1.0, 2.0, 3.0, 4.0), beta = list(1, 2))
/// # Returns the length of alpha (4)
///
/// validate_numeric_args(alpha = c(1.0, 2.0), beta = list(1, 2))
/// # Error: field "alpha" has wrong length: expected 4, got 2
///
/// validate_numeric_args(beta = list(1, 2))
/// # Error: missing required field: "alpha"
/// ```
#[miniextendr]
/// @param dots Named arguments: `alpha` (numeric vector of length 4), `beta` (list), `gamma` (optional character).
pub fn validate_numeric_args(dots: ...) -> Result<i32, String> {
    use miniextendr_api::ffi;

    let args = dots
        .typed(typed_list!(
            alpha => numeric(4),
            beta => list(),
            gamma? => character()
        ))
        .map_err(|e| e.to_string())?;

    // Get the raw SEXP and return its length
    let alpha = args.get_raw("alpha").map_err(|e| e.to_string())?;
    Ok(unsafe { ffi::Rf_xlength(alpha) } as i32)
}

/// Validate dots in strict mode (no extra fields allowed).
///
/// # Example from R
/// ```r
/// validate_strict_args(x = 1.0, y = 2.0)
/// # Returns "x=1.0, y=2.0"
///
/// validate_strict_args(x = 1.0, y = 2.0, z = 3.0)
/// # Error: unexpected extra fields: ["z"]
/// ```
#[miniextendr]
/// @param dots Named arguments: `x` (numeric), `y` (numeric). No extra fields allowed.
pub fn validate_strict_args(dots: ...) -> Result<String, String> {
    let args = dots
        .typed(typed_list!(@exact; x => numeric(), y => numeric()))
        .map_err(|e| e.to_string())?;

    let x: f64 = args.get("x").map_err(|e| e.to_string())?;
    let y: f64 = args.get("y").map_err(|e| e.to_string())?;

    Ok(format!("x={}, y={}", x, y))
}

/// Validate with class checking.
///
/// # Example from R
/// ```r
/// validate_class_args(data = data.frame(a = 1:3))
/// # Returns the number of columns (1)
///
/// validate_class_args(data = list(a = 1:3))
/// # Error: field "data" has wrong type: expected data.frame, got list
/// ```
#[miniextendr]
/// @param dots Named arguments: `data` (data.frame).
pub fn validate_class_args(dots: ...) -> Result<i32, String> {
    use miniextendr_api::ffi;

    let args = dots
        .typed(typed_list!(data => "data.frame"))
        .map_err(|e| e.to_string())?;

    // Data frame is a list of columns, so Rf_xlength returns ncol
    let data = args.get_raw("data").map_err(|e| e.to_string())?;
    let ncol = unsafe { ffi::Rf_xlength(data) };

    Ok(ncol as i32)
}

miniextendr_module! {
    mod dots_tests;

    fn greetings_with_named_dots;
    fn greetings_with_named_and_unused_dots;
    fn greetings_with_nameless_dots;
    fn greetings_last_as_named_dots;
    fn greetings_last_as_named_and_unused_dots;
    fn greetings_last_as_nameless_dots;

    // typed_list examples
    fn validate_numeric_args;
    fn validate_strict_args;
    fn validate_class_args;
}
