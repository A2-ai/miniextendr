//! Tests for R dots (`...`) handling.

use miniextendr_api::{miniextendr, miniextendr_module};

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

miniextendr_module! {
    mod dots_tests;

    fn greetings_with_named_dots;
    fn greetings_with_named_and_unused_dots;
    fn greetings_with_nameless_dots;
    fn greetings_last_as_named_dots;
    fn greetings_last_as_named_and_unused_dots;
    fn greetings_last_as_nameless_dots;
}
