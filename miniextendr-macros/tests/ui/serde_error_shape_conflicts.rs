//! Test: a `SerdeErrorShape` const whose options contradict each other fails
//! const evaluation, so it does not compile (#2207).
//!
//! Each builder method checks its fields against the options already set and
//! panics with a message naming the field; `#[track_caller]` puts the error at
//! the const.

use miniextendr_api::condition::SerdeErrorShape;

/// A field in two `as_*` lists.
pub const TWO_TYPES: SerdeErrorShape = SerdeErrorShape::new()
    .as_character(&["rule"])
    .as_integer(&["n"])
    .as_double(&["n"]);

/// A field both skipped and given a type.
pub const SKIPPED_AND_TYPED: SerdeErrorShape = SerdeErrorShape::new()
    .skip(&["message"])
    .as_character(&["message"]);

fn main() {}
