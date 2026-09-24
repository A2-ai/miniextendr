//! `match_arg` / `choices` on `Either<T, R>`: a choice or a value of another
//! kind. The R formal is `T`'s choice vector; character or factor input is
//! matched and becomes `Left(T)`, anything else is converted to `R`.

use miniextendr_api::either_impl::Either;
use miniextendr_api::{DataFrame, MatchArg, Missing, miniextendr};

/// Route of administration: the choice arm of the fixtures below.
#[derive(Copy, Clone, Debug, PartialEq, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum Route {
    Oral,
    Bolus,
    Infusion,
}

/// `"<Route>"` for a choice, `"frame:<rows>x<cols>"` for a data frame.
fn describe_route(route: Either<Route, DataFrame>) -> String {
    match route {
        Either::Left(route) => format!("{route:?}"),
        Either::Right(frame) => format!("frame:{}x{}", frame.nrow(), frame.ncol()),
    }
}

/// A route name or a data frame of doses.
///
/// @export
#[miniextendr]
pub fn match_arg_either_route(#[miniextendr(match_arg)] route: Either<Route, DataFrame>) -> String {
    describe_route(route)
}

/// An optional route name or data frame: `NULL` is no choice.
///
/// @export
#[miniextendr]
pub fn match_arg_either_route_optional(
    #[miniextendr(match_arg)] maybe_route: Option<Either<Route, DataFrame>>,
) -> String {
    maybe_route.map_or_else(|| "none".to_string(), describe_route)
}

/// A route name or data frame whose omission is reported.
///
/// @export
#[miniextendr]
pub fn match_arg_either_route_omitted(
    #[miniextendr(match_arg)] route: Missing<Either<Route, DataFrame>>,
) -> String {
    match route {
        Missing::Absent => "absent".to_string(),
        Missing::Present(route) => describe_route(route),
    }
}

/// An optional route name or data frame whose omission is reported.
///
/// @export
#[miniextendr]
pub fn match_arg_either_route_omitted_optional(
    #[miniextendr(match_arg)] route: Missing<Option<Either<Route, DataFrame>>>,
) -> String {
    match route {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(route)) => describe_route(route),
    }
}

/// An inline level name or a number.
///
/// @export
#[miniextendr]
pub fn choices_either_level(
    #[miniextendr(choices("low", "mid", "high"))] level: Either<String, f64>,
) -> String {
    match level {
        Either::Left(level) => format!("level:{level}"),
        Either::Right(n) => format!("number:{n}"),
    }
}

/// Plans a route on an R6 object: the `Either` choice on an impl method.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRoutePlanner;

#[miniextendr(r6)]
impl EitherRoutePlanner {
    pub fn new() -> Self {
        EitherRoutePlanner
    }

    /// Describe a route name or a data frame of doses.
    #[miniextendr(match_arg(route))]
    pub fn plan(&self, route: Either<Route, DataFrame>) -> String {
        describe_route(route)
    }
}

/// A trait method taking an inline choice or a number: `choices(...)` on
/// `Either<String, f64>` through the trait-impl codegen path.
#[miniextendr]
pub trait RouteLevel {
    /// Describe a level name or a number.
    fn level(&self, level: Either<String, f64>) -> String;
}

#[miniextendr(r6)]
impl RouteLevel for EitherRoutePlanner {
    #[miniextendr(choices(level = "low, mid, high"))]
    fn level(&self, level: Either<String, f64>) -> String {
        match level {
            Either::Left(level) => format!("level:{level}"),
            Either::Right(n) => format!("number:{n}"),
        }
    }
}
