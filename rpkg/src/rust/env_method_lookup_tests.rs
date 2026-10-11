//! `obj$name` lookup on an env class whose traits share member names (#1677),
//! driven by `tests/testthat/test-env-method-binding.R`.
//!
//! `LookupLeft` and `LookupRight` both provide an instance method, a static
//! method and a constant of the same names, so a flat `obj$side()` must refuse
//! to pick one. `label` is also an inherent method, which wins. Every member
//! returns which impl ran, so a wrapper that called the wrong one shows.

use miniextendr_api::miniextendr;

/// Env class implementing two traits with overlapping member names.
#[derive(miniextendr_api::ExternalPtr)]
pub struct LookupHost {
    value: i32,
}

#[miniextendr(env, noexport)]
impl LookupHost {
    /// Create a host holding `value`.
    /// @param value The host's value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// Named like a member of both traits.
    pub fn label(&self) -> String {
        "inherent".to_string()
    }
}

/// First of two traits whose members share names.
#[miniextendr]
pub trait LookupLeft {
    /// Shared with `LookupRight`.
    const LIMIT: i32;
    /// Shared with `LookupRight`.
    fn side(&self) -> String;
    /// Shared with `LookupRight`.
    fn origin() -> String;
    /// Shared with `LookupRight` and an inherent method of `LookupHost`.
    fn label(&self) -> String;
    /// Only this trait provides it.
    fn left_only(&self) -> i32;
}

/// Second of two traits whose members share names.
#[miniextendr]
pub trait LookupRight {
    /// Shared with `LookupLeft`.
    const LIMIT: i32;
    /// Shared with `LookupLeft`.
    fn side(&self) -> String;
    /// Shared with `LookupLeft`.
    fn origin() -> String;
    /// Shared with `LookupLeft` and an inherent method of `LookupHost`.
    fn label(&self) -> String;
}

#[miniextendr(env, noexport)]
impl LookupLeft for LookupHost {
    const LIMIT: i32 = 10;

    fn side(&self) -> String {
        format!("left {}", self.value)
    }

    fn origin() -> String {
        "left".to_string()
    }

    fn label(&self) -> String {
        "left label".to_string()
    }

    fn left_only(&self) -> i32 {
        self.value + 1
    }
}

#[miniextendr(env, noexport)]
impl LookupRight for LookupHost {
    const LIMIT: i32 = 20;

    fn side(&self) -> String {
        format!("right {}", self.value)
    }

    fn origin() -> String {
        "right".to_string()
    }

    fn label(&self) -> String {
        "right label".to_string()
    }
}
