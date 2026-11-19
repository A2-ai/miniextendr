//! Test: MatchArg needs at least one variant; the error points at the name.

use miniextendr_macros::MatchArg;

/// No choices to match.
#[derive(MatchArg)]
pub enum Empty {}

fn main() {}
