//! Test: a hand-written `TryFromSexp` that declares `CHARACTER_ONLY` (here by
//! forwarding `String`'s) is rejected as the `R` arm of an `Either` choice
//! parameter.

use miniextendr_api::either_impl::Either;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::{MatchArg, SEXP, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Route {
    Oral,
    Bolus,
}

pub struct Username(String);

impl TryFromSexp for Username {
    type Error = SexpError;
    const CHARACTER_ONLY: bool = <String as TryFromSexp>::CHARACTER_ONLY;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        String::try_from_sexp(sexp).map(Username)
    }
}

#[miniextendr]
pub fn route_or_user(#[miniextendr(match_arg)] route: Either<Route, Username>) -> bool {
    route.is_left()
}

fn main() {}
