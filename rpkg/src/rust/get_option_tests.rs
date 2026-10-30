//! `get_option` fixtures for `test-get-option.R`.

use miniextendr_api::condition::RError;
use miniextendr_api::{get_option, miniextendr};

// region: get_option

/// The option `name` read as an integer, `NA` when unset (a scalar `None`).
/// @param name The option's name.
#[miniextendr(noexport)]
pub fn get_option_int(name: &str) -> Result<Option<i32>, RError> {
    get_option(name)
}

/// The option `name` read as a character vector, `NULL` when unset.
/// @param name The option's name.
#[miniextendr(noexport)]
pub fn get_option_strings(name: &str) -> Result<Option<Vec<String>>, RError> {
    get_option(name)
}

// endregion
