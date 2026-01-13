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
/// @param ... Additional arguments (captured as dots).
pub fn greetings_with_named_dots(dots: ...) {
    let _ = dots;
}

/// @noRd
#[miniextendr]
pub fn greetings_with_named_and_unused_dots(_dots: ...) {}

/// @noRd
#[miniextendr]
pub fn greetings_with_nameless_dots(...) {}

// LIMITATION: Good!
// #[miniextendr]
// fn greetings_with_dots_then_arg(dots: ..., exclamations: i32) {}

/// @noRd
#[miniextendr]
pub fn greetings_last_as_named_and_unused_dots(_exclamations: i32, _dots: ...) {}

/// @noRd
#[miniextendr]
pub fn greetings_last_as_named_dots(_exclamations: i32, dots: ...) {
    let _ = dots;
}

/// @noRd
#[miniextendr]
pub fn greetings_last_as_nameless_dots(_exclamations: i32, ...) {}

// =============================================================================
// typed_list! macro examples
// =============================================================================

/// @noRd
#[miniextendr]
/// @param ... Named arguments: `alpha` (numeric vector of length 4), `beta` (list), `gamma` (optional character).
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

/// @noRd
#[miniextendr]
/// @param ... Named arguments: `x` (numeric), `y` (numeric). No extra fields allowed.
pub fn validate_strict_args(dots: ...) -> Result<String, String> {
    let args = dots
        .typed(typed_list!(@exact; x => numeric(), y => numeric()))
        .map_err(|e| e.to_string())?;

    let x: f64 = args.get("x").map_err(|e| e.to_string())?;
    let y: f64 = args.get("y").map_err(|e| e.to_string())?;

    Ok(format!("x={}, y={}", x, y))
}

/// @noRd
#[miniextendr]
/// @param ... Named arguments: `data` (data.frame).
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

// =============================================================================
// Attribute sugar for typed_list validation
// =============================================================================

/// @noRd
#[miniextendr(dots = typed_list!(x => numeric(), y => numeric()))]
/// @param ... Named arguments: `x` (numeric), `y` (numeric).
pub fn validate_with_attribute(...) -> String {
    // dots_typed is automatically created by the attribute
    let x: f64 = dots_typed.get("x").expect("x");
    let y: f64 = dots_typed.get("y").expect("y");
    format!("x={}, y={}", x, y)
}

/// @noRd
#[miniextendr(dots = typed_list!(name => character(), greeting? => character()))]
/// @param ... Named arguments: `name` (character), `greeting` (optional character).
pub fn validate_attr_optional(...) -> String {
    let name: String = dots_typed.get("name").expect("name");
    let greeting: Option<String> = dots_typed.get_opt("greeting").expect("greeting");
    let greeting = greeting.unwrap_or_else(|| "Hello".to_string());
    format!("{}, {}!", greeting, name)
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

    // attribute sugar examples
    fn validate_with_attribute;
    fn validate_attr_optional;
}
