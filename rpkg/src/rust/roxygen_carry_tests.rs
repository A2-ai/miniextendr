//! Doc comments carry their blank lines and indentation into R help.
//!
//! `man/roxygen_carry_tests.Rd` pins the result, and
//! `tests/testthat/test-roxygen-carry.R` reads it back.

use miniextendr_api::miniextendr;

/// Roxygen carry demo.
///
/// The leading prose becomes the description. This paragraph holds a list:
/// - `first`: an item
///   continued on an indented line
/// - `second`: another item
///
/// @title Roxygen carry demo
/// @details The first paragraph of the details.
///
/// The second paragraph of the details.
/// @return The number 3.
///
///   The second paragraph of the return value, indented.
/// @examples
/// x <- c(1, 2) |>
///   sum()
/// stopifnot(x == roxygen_carry_demo())
#[miniextendr]
pub fn roxygen_carry_demo() -> f64 {
    3.0
}
