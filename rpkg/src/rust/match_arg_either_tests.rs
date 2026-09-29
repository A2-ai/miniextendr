//! `match_arg` / `choices` on `Either<T, R>`: a choice or a value of another
//! kind. The R formal is `T`'s choice vector; character or factor input is
//! matched and becomes `Left(T)`, anything else is converted to `R`. With
//! `several_ok`, `Either<Vec<T>, R>` / `Either<Box<[T]>, R>` take one or more
//! choices or a value of another kind (#1612).

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

/// A route name or a number, without `match_arg`: `TryFromSexp for Either`
/// tries `Route` first, then `f64`. A value that is neither a string nor a
/// number is refused by both, and the argument error names both.
/// @param x A route name or a number.
/// @noRd
#[miniextendr(noexport)]
pub fn either_route_or_number(x: Either<Route, f64>) -> String {
    match x {
        Either::Left(route) => format!("{route:?}"),
        Either::Right(n) => format!("number:{n}"),
    }
}

/// `TRUE or FALSE`, or a route name or a number: a nested `Either` whose
/// arms both refuse a value counts as one arm that refused it.
/// @param x A logical, a route name or a number.
/// @noRd
#[miniextendr(noexport)]
pub fn either_flag_or_route_or_number(x: Either<bool, Either<Route, f64>>) -> String {
    match x {
        Either::Left(flag) => format!("flag:{flag}"),
        Either::Right(inner) => either_route_or_number(inner),
    }
}

/// A route name or a mode name, each through its own `TryFromSexp`: two
/// choice lists read as one.
/// @param x A route name or a mode name.
/// @noRd
#[miniextendr(noexport)]
pub fn either_route_or_mode(x: Either<Route, crate::match_arg_tests::Mode>) -> String {
    match x {
        Either::Left(route) => format!("{route:?}"),
        Either::Right(mode) => format!("mode:{mode:?}"),
    }
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

// region: `several_ok` choice lists with another kind of value (#1612)
//
// `Either<Vec<T>, R>` / `Either<Box<[T]>, R>`: character or factor input is
// matched element by element and becomes `Left(Vec<T>)` (an omitted argument
// selects every choice), anything else, `NULL` included, is converted to `R`.
// The `Missing` / `Option` stacks over the `Either` follow the scalar rule.
// Parameter names are new so the standalone functions do not replace the
// `@param` lines of the scalar fixtures on the shared Rd page.

/// `"Oral,Bolus"` for a list of routes, `"frame:<rows>x<cols>"` for a data
/// frame.
fn describe_routes(routes: Either<Vec<Route>, DataFrame>) -> String {
    match routes {
        Either::Left(routes) => routes
            .iter()
            .map(|route| format!("{route:?}"))
            .collect::<Vec<_>>()
            .join(","),
        Either::Right(frame) => format!("frame:{}x{}", frame.nrow(), frame.ncol()),
    }
}

/// `tiers:<a>,<b>` for a list of tier names, `number:<n>` for a number.
fn describe_tiers(tiers: Either<Vec<String>, f64>) -> String {
    match tiers {
        Either::Left(tiers) => format!("tiers:{}", tiers.join(",")),
        Either::Right(n) => format!("number:{n}"),
    }
}

/// `routes=<..>;tiers=<..>`: the routes as `describe_routes` reports them and
/// the tiers as `describe_tiers` does, or `absent` for omitted tiers.
fn describe_routes_tiers(
    routes: Either<Vec<Route>, DataFrame>,
    tiers: Missing<Either<Vec<String>, f64>>,
) -> String {
    let tiers = tiers
        .into_option()
        .map_or_else(|| "absent".to_string(), describe_tiers);
    format!("routes={};tiers={tiers}", describe_routes(routes))
}

/// Route names or a data frame of doses.
///
/// @export
#[miniextendr]
pub fn match_arg_either_routes(
    #[miniextendr(match_arg, several_ok)] routes: Either<Vec<Route>, DataFrame>,
) -> String {
    describe_routes(routes)
}

/// Route names or a data frame of doses, as a boxed slice.
///
/// @export
#[miniextendr]
pub fn match_arg_either_routes_boxed(
    #[miniextendr(match_arg, several_ok)] route_set: Either<Box<[Route]>, DataFrame>,
) -> String {
    describe_routes(route_set.map_left(Vec::from))
}

/// Route names or a data frame whose omission is reported.
///
/// @export
#[miniextendr]
pub fn match_arg_either_routes_omitted(
    #[miniextendr(match_arg, several_ok)] route_list: Missing<Either<Vec<Route>, DataFrame>>,
) -> String {
    route_list
        .into_option()
        .map_or_else(|| "absent".to_string(), describe_routes)
}

/// Optional route names or a data frame: `NULL` is no choice.
///
/// @export
#[miniextendr]
pub fn match_arg_either_routes_optional(
    #[miniextendr(match_arg, several_ok)] maybe_routes: Option<Either<Vec<Route>, DataFrame>>,
) -> String {
    maybe_routes.map_or_else(|| "none".to_string(), describe_routes)
}

/// Optional route names or a data frame whose omission is reported.
///
/// @export
#[miniextendr]
pub fn match_arg_either_routes_omitted_optional(
    #[miniextendr(match_arg, several_ok)] route_pick: Missing<
        Option<Either<Vec<Route>, DataFrame>>,
    >,
) -> String {
    match route_pick {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(routes)) => describe_routes(routes),
    }
}

/// Inline tier names or a number.
///
/// @export
#[miniextendr]
pub fn choices_either_tiers(
    #[miniextendr(choices("low", "mid", "high"), several_ok)] tiers: Either<Vec<String>, f64>,
) -> String {
    describe_tiers(tiers)
}

/// An env class whose method takes `several_ok` `Either` choice lists.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRoutesEnv;

#[miniextendr(env)]
impl EitherRoutesEnv {
    pub fn new() -> Self {
        EitherRoutesEnv
    }

    /// Route names or a data frame of doses, and omittable tier names or a
    /// number.
    #[miniextendr(
        match_arg_several_ok(routes),
        choices_several_ok(tiers = "low, mid, high")
    )]
    pub fn plan(
        &self,
        routes: Either<Vec<Route>, DataFrame>,
        tiers: Missing<Either<Vec<String>, f64>>,
    ) -> String {
        describe_routes_tiers(routes, tiers)
    }
}

/// An R6 class whose method takes `several_ok` `Either` choice lists.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRoutesR6;

#[miniextendr(r6)]
impl EitherRoutesR6 {
    pub fn new() -> Self {
        EitherRoutesR6
    }

    /// Route names or a data frame of doses, and omittable tier names or a
    /// number.
    #[miniextendr(
        match_arg_several_ok(routes),
        choices_several_ok(tiers = "low, mid, high")
    )]
    pub fn plan(
        &self,
        routes: Either<Vec<Route>, DataFrame>,
        tiers: Missing<Either<Vec<String>, f64>>,
    ) -> String {
        describe_routes_tiers(routes, tiers)
    }
}

/// A trait method taking inline tier names or a number: `choices_several_ok`
/// on `Either<Vec<String>, f64>` through the trait-impl codegen path.
#[miniextendr]
pub trait RouteTiers {
    /// Describe tier names or a number.
    fn tiers(&self, tiers: Either<Vec<String>, f64>) -> String;
}

#[miniextendr(r6)]
impl RouteTiers for EitherRoutesR6 {
    #[miniextendr(choices_several_ok(tiers = "low, mid, high"))]
    fn tiers(&self, tiers: Either<Vec<String>, f64>) -> String {
        describe_tiers(tiers)
    }
}

/// An S3 class whose method takes `several_ok` `Either` choice lists.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRoutesS3;

#[miniextendr(s3)]
impl EitherRoutesS3 {
    pub fn new() -> Self {
        EitherRoutesS3
    }

    /// Route names or a data frame of doses, and omittable tier names or a
    /// number.
    #[miniextendr(
        match_arg_several_ok(routes),
        choices_several_ok(tiers = "low, mid, high")
    )]
    pub fn plan_routes_s3(
        &self,
        routes: Either<Vec<Route>, DataFrame>,
        tiers: Missing<Either<Vec<String>, f64>>,
    ) -> String {
        describe_routes_tiers(routes, tiers)
    }
}

/// An S4 class whose method takes `several_ok` `Either` choice lists.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRoutesS4;

#[miniextendr(s4)]
impl EitherRoutesS4 {
    pub fn new() -> Self {
        EitherRoutesS4
    }

    /// Route names or a data frame of doses, and omittable tier names or a
    /// number.
    #[miniextendr(
        match_arg_several_ok(routes),
        choices_several_ok(tiers = "low, mid, high")
    )]
    pub fn plan_routes(
        &self,
        routes: Either<Vec<Route>, DataFrame>,
        tiers: Missing<Either<Vec<String>, f64>>,
    ) -> String {
        describe_routes_tiers(routes, tiers)
    }
}

/// An S7 class whose constructor and method take `several_ok` `Either` choice
/// lists. The constructor records what it was given.
#[derive(miniextendr_api::ExternalPtr)]
pub struct EitherRoutesS7 {
    given: String,
}

#[miniextendr(s7)]
impl EitherRoutesS7 {
    #[miniextendr(match_arg_several_ok(start))]
    pub fn new(start: Missing<Either<Vec<Route>, DataFrame>>) -> Self {
        let given = start
            .into_option()
            .map_or_else(|| "absent".to_string(), describe_routes);
        EitherRoutesS7 { given }
    }

    /// What the constructor was given.
    pub fn routes_given(&self) -> String {
        self.given.clone()
    }

    /// Route names or a data frame of doses, and omittable tier names or a
    /// number.
    #[miniextendr(
        match_arg_several_ok(routes),
        choices_several_ok(tiers = "low, mid, high")
    )]
    pub fn plan_routes_s7(
        &self,
        routes: Either<Vec<Route>, DataFrame>,
        tiers: Missing<Either<Vec<String>, f64>>,
    ) -> String {
        describe_routes_tiers(routes, tiers)
    }
}

/// vctrs fixture whose constructor and static method take `several_ok`
/// `Either` choice lists. The payload is each route's position for route names
/// and one zero per row for a data frame.
pub struct EitherRoutesVctrs;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "routes"))]
impl EitherRoutesVctrs {
    #[allow(clippy::new_ret_no_self)]
    #[miniextendr(match_arg_several_ok(routes))]
    pub fn new(routes: Either<Vec<Route>, DataFrame>) -> Vec<f64> {
        match routes {
            Either::Left(routes) => routes
                .iter()
                .map(|route| match route {
                    Route::Oral => 1.0,
                    Route::Bolus => 2.0,
                    Route::Infusion => 3.0,
                })
                .collect(),
            Either::Right(frame) => vec![0.0; frame.nrow()],
        }
    }

    /// Route names or a data frame of doses, and omittable tier names or a
    /// number.
    #[miniextendr(
        match_arg_several_ok(routes),
        choices_several_ok(tiers = "low, mid, high")
    )]
    pub fn plan_routes(
        routes: Either<Vec<Route>, DataFrame>,
        tiers: Missing<Either<Vec<String>, f64>>,
    ) -> String {
        describe_routes_tiers(routes, tiers)
    }
}

/// Omittable inline grade names or a number through the trait-method codegen
/// path (`choices_several_ok(p = "...")` on a string list).
#[miniextendr]
pub trait EitherGrades {
    /// `absent`, or the grades as `describe_tiers` reports them.
    fn either_grades(&self, grades: Missing<Either<Vec<String>, f64>>) -> String;
}

/// `absent` / `describe_tiers`'s report.
fn describe_layered_grades(grades: Missing<Either<Vec<String>, f64>>) -> String {
    grades
        .into_option()
        .map_or_else(|| "absent".to_string(), describe_tiers)
}

#[miniextendr(s3)]
impl EitherGrades for EitherRoutesS3 {
    #[miniextendr(choices_several_ok(grades = "low, mid, high"))]
    fn either_grades(&self, grades: Missing<Either<Vec<String>, f64>>) -> String {
        describe_layered_grades(grades)
    }
}

#[miniextendr(s7)]
impl EitherGrades for EitherRoutesS7 {
    #[miniextendr(choices_several_ok(grades = "low, mid, high"))]
    fn either_grades(&self, grades: Missing<Either<Vec<String>, f64>>) -> String {
        describe_layered_grades(grades)
    }
}

// endregion
