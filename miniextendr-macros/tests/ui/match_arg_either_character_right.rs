//! Test: the choice check of an `Either<T, R>` choice parameter sends every
//! character or factor argument to `T`, so an `R` that reads only character
//! input (`String`) could receive at most `NULL`. The macro rejects it, naming
//! the parameter and the arm.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{MatchArg, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Route {
    Oral,
    Bolus,
}

#[miniextendr]
pub fn route_or_text(#[miniextendr(match_arg)] route: Either<Route, String>) -> bool {
    route.is_left()
}

fn main() {}
