//! Test: MatchArg can only be applied to enums. The error points at the
//! `struct` keyword, not at the doc comment or `pub` ahead of it.

use miniextendr_macros::MatchArg;

/// A struct is not a choice.
#[derive(MatchArg)]
pub struct Bad {
    x: i32,
}

fn main() {}
