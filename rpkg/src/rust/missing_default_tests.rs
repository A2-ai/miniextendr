//! A `default = "..."` on a `Missing<T>` parameter (#2161). The formal shows
//! the default, an omitted argument still reaches Rust as `Missing::Absent`,
//! and every supplied value is `Present`, the default's own value included.
//! The default is never evaluated: the wrapper tests `missing()` before it
//! reads the argument.
//!
//! These fixtures have their own file, so their own help page: the omission
//! note on their `@param` lines is checked there.

use miniextendr_api::{Missing, miniextendr};

/// `absent`, or the value.
fn describe_tol(tol: Missing<f64>) -> String {
    match tol {
        Missing::Absent => "absent".to_string(),
        Missing::Present(tol) => format!("{tol}"),
    }
}

// region: Standalone functions

/// A tolerance whose default shows in the usage line; omitting it is
/// `Absent`.
///
/// @export
#[miniextendr]
pub fn missing_default_tol(#[miniextendr(default = "1e-8")] tol: Missing<f64>) -> String {
    describe_tol(tol)
}

/// Optional weights with a `NULL` default: an omitted argument is `Absent`,
/// an explicit `NULL` is `Present(None)`.
///
/// @export
#[miniextendr]
pub fn missing_default_weights(
    #[miniextendr(default = "NULL")] weights: Missing<Option<Vec<f64>>>,
) -> String {
    match weights {
        Missing::Absent => "absent".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(w)) => format!("n={}", w.len()),
    }
}

/// A default that fails when evaluated, next to an R-side check: omitting
/// the argument evaluates neither.
///
/// @param x A number, not NA.
/// @export
#[miniextendr]
pub fn missing_default_unforced(
    #[miniextendr(default = "stop(\"the default was evaluated\")", no_na)] x: Missing<f64>,
) -> String {
    describe_tol(x)
}
// endregion

// region: Methods

/// An S4 class whose method takes a `Missing<f64>` with a default.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MissingDefaultS4;

#[miniextendr(s4)]
impl MissingDefaultS4 {
    pub fn new() -> Self {
        MissingDefaultS4
    }

    /// `absent`, or the tolerance given.
    #[miniextendr(defaults(tol = "1e-8"))]
    pub fn tol(&self, tol: Missing<f64>) -> String {
        describe_tol(tol)
    }
}

/// An R6 class whose method takes a `Missing<f64>` with a default.
#[derive(miniextendr_api::ExternalPtr)]
pub struct MissingDefaultR6;

#[miniextendr(r6)]
impl MissingDefaultR6 {
    pub fn new() -> Self {
        MissingDefaultR6
    }

    /// `absent`, or the tolerance given.
    #[miniextendr(defaults(tol = "1e-8"))]
    pub fn tol(&self, tol: Missing<f64>) -> String {
        describe_tol(tol)
    }
}

/// A trait method taking a `Missing<f64>`; its impl gives the default.
#[miniextendr]
pub trait MissingDefaultTol {
    /// `absent`, or the tolerance given.
    fn trait_tol(&self, tol: Missing<f64>) -> String;
}

#[miniextendr(r6)]
impl MissingDefaultTol for MissingDefaultR6 {
    #[miniextendr(defaults(tol = "1e-8"))]
    fn trait_tol(&self, tol: Missing<f64>) -> String {
        describe_tol(tol)
    }
}
// endregion
