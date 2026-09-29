//! Test: a `#[derive(MatchArg)]` enum reads character or factor input (and
//! `NULL` as its first choice), so as the `R` arm of an `Either` choice
//! parameter it is reachable only through `NULL` and is rejected.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{MatchArg, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Route {
    Oral,
    Bolus,
}

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Unit {
    Mg,
    Ug,
}

#[miniextendr]
pub fn route_or_unit(#[miniextendr(match_arg)] route: Either<Route, Unit>) -> bool {
    route.is_left()
}

fn main() {}
