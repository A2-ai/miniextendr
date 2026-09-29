//! Test: a `several_ok` choice list with another kind of value,
//! `Either<Vec<T>, R>`, rejects an `R` that reads only character input
//! (`Vec<String>`): the choice check takes every character argument.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{MatchArg, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Route {
    Oral,
    Bolus,
}

#[miniextendr]
pub fn routes_or_texts(
    #[miniextendr(match_arg, several_ok)] routes: Either<Vec<Route>, Vec<String>>,
) -> bool {
    routes.is_left()
}

fn main() {}
