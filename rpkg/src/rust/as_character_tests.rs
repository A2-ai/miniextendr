//! Test fixtures for the `as.character()`-style AsCharacter / AsCharacterVec
//! markers, driven by `tests/testthat/test-as-character.R`.

use miniextendr_api::{AsCharacter, AsCharacterVec, miniextendr};
use std::collections::HashSet;

// region: AsCharacter / AsCharacterVec — any atomic vector, factor labels → strings (NA → None)

#[miniextendr(noexport)]
pub fn test_as_character_vec(x: AsCharacterVec) -> Vec<Option<String>> {
    x.0
}

#[miniextendr(noexport)]
pub fn test_as_character(x: AsCharacter) -> Option<String> {
    x.0
}

/// `NULL` and `NA` both come back as `NA_character_`.
#[miniextendr(noexport)]
pub fn test_as_character_opt(x: Option<AsCharacter>) -> Option<String> {
    x.and_then(|v| v.0)
}

/// The number of distinct non-NA labels: the identifier-counting use the
/// marker is for.
#[miniextendr(noexport)]
pub fn test_as_character_n_distinct(x: AsCharacterVec) -> i32 {
    let distinct: HashSet<String> = x.0.into_iter().flatten().collect();
    i32::try_from(distinct.len()).expect("distinct label count fits i32")
}

/// With `no_na`, R refuses `NA` before the call, so every value is present.
#[miniextendr(noexport)]
pub fn test_as_character_no_na(#[miniextendr(no_na)] x: AsCharacterVec) -> Vec<String> {
    x.0.into_iter()
        .map(|v| v.expect("no_na refuses NA in R"))
        .collect()
}

// endregion
