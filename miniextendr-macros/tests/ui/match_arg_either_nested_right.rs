//! Test: a nested `Either` on the `R` side is checked leaf by leaf. `f64`
//! reads numbers and passes; `String` would only ever see `NULL` and is named
//! inside the nested `Either`.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{MatchArg, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Route {
    Oral,
    Bolus,
}

#[miniextendr]
pub fn route_or_number_or_text(
    #[miniextendr(match_arg)] route: Either<Route, Either<f64, String>>,
) -> bool {
    route.is_left()
}

fn main() {}
