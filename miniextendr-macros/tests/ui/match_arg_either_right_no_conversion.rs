//! Test: an `R` arm without `TryFromSexp` gets exactly one error, the
//! decode's unmet bound. The character-input check reads the arm through a
//! probe that falls back to `false`, so it adds no second error.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{MatchArg, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Route {
    Oral,
    Bolus,
}

pub struct NotConvertible;

#[miniextendr]
pub fn route_or_opaque(#[miniextendr(match_arg)] route: Either<Route, NotConvertible>) -> bool {
    route.is_left()
}

fn main() {}
