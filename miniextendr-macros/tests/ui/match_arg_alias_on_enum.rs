//! Test: `#[match_arg(alias = "...")]` spells one choice, so it goes on that
//! choice's variant, not on the enum (#1843).

use miniextendr_api::MatchArg;

#[derive(Copy, Clone, MatchArg)]
#[match_arg(alias = "grey")]
enum Shade {
    Red,
    Gray,
}

fn main() {}
