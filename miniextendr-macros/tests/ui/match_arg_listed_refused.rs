//! Test: `#[match_arg(alias = "...", listed = ...)]` takes `true` or `false`
//! only, and applies to the aliases of its own attribute, so one without an
//! alias there is refused too (#2148).

use miniextendr_api::MatchArg;

#[derive(Copy, Clone, MatchArg)]
#[match_arg(rename_all = "lower")]
enum Route {
    #[match_arg(alias = "oral", listed = "no")]
    Extravascular,
    IvBolus,
}

#[derive(Copy, Clone, MatchArg)]
#[match_arg(rename_all = "lower")]
enum Shade {
    #[match_arg(alias = "grey")]
    #[match_arg(listed = false)]
    Gray,
    Red,
}

fn main() {}
