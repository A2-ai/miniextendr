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

// region: `Either` choices on S3 / S4 / S7 / vctrs methods and on trait methods
//
// The class systems the R6 fixture above does not reach. Each method takes a
// `match_arg` route (`Either<Route, DataFrame>`) and an omittable inline level
// (`choices(...)` on `Missing<Either<String, f64>>`) and reports both through
// `describe_route_level`.

/// `level:<name>` for a level name, `number:<n>` for a number.
fn describe_level(level: Either<String, f64>) -> String {
    match level {
        Either::Left(level) => format!("level:{level}"),
        Either::Right(n) => format!("number:{n}"),
    }
}

/// `route=<..>;level=<..>`: the route as `describe_route` reports it and the
/// level as `describe_level` does, or `absent` for an omitted level.
fn describe_route_level(
    route: Either<Route, DataFrame>,
    level: Missing<Either<String, f64>>,
) -> String {
    let level = level
        .into_option()
        .map_or_else(|| "absent".to_string(), describe_level);
    format!("route={};level={level}", describe_route(route))
}

/// An S3 class whose method takes `Either` choice parameters.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRouteS3;

#[miniextendr(s3)]
impl EitherRouteS3 {
    pub fn new() -> Self {
        EitherRouteS3
    }

    /// A route name or a data frame of doses, and an omittable level name or
    /// number.
    #[miniextendr(match_arg(route), choices(level = "low, mid, high"))]
    pub fn plan_s3(
        &self,
        route: Either<Route, DataFrame>,
        level: Missing<Either<String, f64>>,
    ) -> String {
        describe_route_level(route, level)
    }
}

/// An S4 class whose method takes `Either` choice parameters.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRouteS4;

#[miniextendr(s4)]
impl EitherRouteS4 {
    pub fn new() -> Self {
        EitherRouteS4
    }

    /// A route name or a data frame of doses, and an omittable level name or
    /// number.
    #[miniextendr(match_arg(route), choices(level = "low, mid, high"))]
    pub fn plan(
        &self,
        route: Either<Route, DataFrame>,
        level: Missing<Either<String, f64>>,
    ) -> String {
        describe_route_level(route, level)
    }
}

/// An S7 class whose constructor and method take `Either` choice parameters.
/// The constructor records what it was given.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRouteS7 {
    given: String,
}

#[miniextendr(s7)]
impl EitherRouteS7 {
    #[miniextendr(match_arg(start))]
    pub fn new(start: Missing<Either<Route, DataFrame>>) -> Self {
        let given = start
            .into_option()
            .map_or_else(|| "absent".to_string(), describe_route);
        EitherRouteS7 { given }
    }

    /// What the constructor was given.
    pub fn route_given(&self) -> String {
        self.given.clone()
    }

    /// A route name or a data frame of doses, and an omittable level name or
    /// number.
    #[miniextendr(match_arg(route), choices(level = "low, mid, high"))]
    pub fn plan_s7(
        &self,
        route: Either<Route, DataFrame>,
        level: Missing<Either<String, f64>>,
    ) -> String {
        describe_route_level(route, level)
    }
}

/// vctrs fixture whose constructor and static method take `Either` choice
/// parameters. The payload is the route's position for a choice and one zero
/// per row for a data frame.
pub struct EitherRouteVctrs;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "route"))]
impl EitherRouteVctrs {
    /// @param route One of "oral", "bolus", "infusion", or a data frame.
    #[allow(clippy::new_ret_no_self)]
    #[miniextendr(match_arg(route))]
    pub fn new(route: Either<Route, DataFrame>) -> Vec<f64> {
        match route {
            Either::Left(Route::Oral) => vec![1.0],
            Either::Left(Route::Bolus) => vec![2.0],
            Either::Left(Route::Infusion) => vec![3.0],
            Either::Right(frame) => vec![0.0; frame.nrow()],
        }
    }

    /// A route name or a data frame of doses, and an omittable level name or
    /// number.
    #[miniextendr(match_arg(route), choices(level = "low, mid, high"))]
    pub fn plan(route: Either<Route, DataFrame>, level: Missing<Either<String, f64>>) -> String {
        describe_route_level(route, level)
    }
}

/// An omittable inline choice or a number through the trait-method codegen
/// path (`choices(p = "...")` on a string type). A trait's parameter types
/// also cross the trait ABI, which needs `TryFromSexp` and `IntoR` for them,
/// so the `Option<Either<..>>` layer (which has neither) is not available
/// here.
#[miniextendr]
pub trait EitherGrade {
    /// `absent`, or the grade as `describe_level` reports it.
    fn either_grade(&self, grade: Missing<Either<String, f64>>) -> String;
}

/// `absent` / `describe_level`'s report.
fn describe_layered_grade(grade: Missing<Either<String, f64>>) -> String {
    grade
        .into_option()
        .map_or_else(|| "absent".to_string(), describe_level)
}

#[miniextendr(s3)]
impl EitherGrade for EitherRouteS3 {
    #[miniextendr(choices(grade = "low, mid, high"))]
    fn either_grade(&self, grade: Missing<Either<String, f64>>) -> String {
        describe_layered_grade(grade)
    }
}

#[miniextendr(s7)]
impl EitherGrade for EitherRouteS7 {
    #[miniextendr(choices(grade = "low, mid, high"))]
    fn either_grade(&self, grade: Missing<Either<String, f64>>) -> String {
        describe_layered_grade(grade)
    }
}

// endregion
