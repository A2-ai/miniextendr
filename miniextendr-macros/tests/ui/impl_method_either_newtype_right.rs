//! Test: on an impl method, an `Either` choice parameter whose `R` arm is a
//! `#[derive(TryFromSexp)]` newtype of `String` is rejected: the derive
//! forwards the inner type's `CHARACTER_ONLY`.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{ExternalPtr, MatchArg, TryFromSexp, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Route {
    Oral,
    Bolus,
}

#[derive(TryFromSexp)]
pub struct Tag(String);

#[derive(ExternalPtr)]
pub struct Dosing;

#[miniextendr(env)]
impl Dosing {
    pub fn new() -> Self {
        Dosing
    }

    #[miniextendr(match_arg(route))]
    pub fn give(&self, route: Either<Route, Tag>) -> bool {
        route.is_left()
    }
}

fn main() {}
