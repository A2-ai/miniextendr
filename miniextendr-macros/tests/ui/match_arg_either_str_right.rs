//! Test: a borrowed string arm (`&str`, converted as `&'static str`) is
//! rejected like `String`: the check reads the arm's `TryFromSexp` through the
//! probe with the lifetime inferred.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{MatchArg, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Route {
    Oral,
    Bolus,
}

#[miniextendr]
pub fn route_or_str(#[miniextendr(match_arg)] route: Either<Route, &str>) -> bool {
    route.is_left()
}

fn main() {}
