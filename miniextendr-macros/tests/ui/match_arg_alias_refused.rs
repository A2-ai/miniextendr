//! Test: `#[derive(MatchArg)]` refuses an alias that is empty, `"NA"`, a
//! choice, given twice, or a prefix of another variant's choice, each at its
//! literal and all in one pass (#1843).

use miniextendr_api::MatchArg;

#[derive(Copy, Clone, MatchArg)]
#[match_arg(rename_all = "lower")]
enum Shade {
    #[match_arg(alias = "")]
    Red,
    #[match_arg(alias = "NA", alias = "blue", alias = "grey")]
    Gray,
    #[match_arg(alias = "grey")]
    Blue,
    #[match_arg(alias = "gr")]
    Green,
}

fn main() {}
