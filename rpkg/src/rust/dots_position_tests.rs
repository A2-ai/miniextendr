//! Fixtures for a `&Dots` parameter at any position in the signature.
//!
//! The parameter of type `&Dots` is R's `...` where it sits: the R formals
//! and the `.Call()` arguments follow the Rust signature order, and every
//! formal after the dots is matched by exact name only.

use miniextendr_api::dots::Dots;
use miniextendr_api::{miniextendr, typed_list};

/// Number of values captured in the dots.
fn dots_count(rest: &Dots) -> i32 {
    i32::try_from(rest.len()).expect("fewer than 2^31 dots")
}

// region: standalone functions

/// Report `x`, the number of dots and `overwrite`, for a formal after `...`.
/// @param x A number.
/// @param ... Values counted by Rust.
/// @param overwrite Whether to overwrite; matched by its full name only.
/// @return A string `"x=<x> dots=<n> overwrite=<overwrite>"`.
/// @examples
/// dots_mid(1L, 2, 3, overwrite = TRUE)
#[miniextendr]
pub fn dots_mid(x: i32, rest: &Dots, #[miniextendr(default = "FALSE")] overwrite: bool) -> String {
    format!("x={x} dots={} overwrite={overwrite}", dots_count(rest))
}

/// The dots first: every formal is matched by name.
/// @param ... Values counted by Rust.
/// @param x A number.
#[miniextendr(noexport)]
pub fn dots_first(rest: &Dots, x: i32) -> String {
    format!("dots={} x={x}", dots_count(rest))
}

/// `typed_list!` sugar on a middle `&Dots`.
/// @param x A number.
/// @param ... `a`, a number.
/// @param flag A flag.
#[miniextendr(noexport, dots = typed_list!(a => numeric()))]
pub fn dots_mid_typed(x: i32, rest: &Dots, #[miniextendr(default = "FALSE")] flag: bool) -> f64 {
    let _ = rest;
    let a: f64 = dots_typed.get("a").expect("a");
    f64::from(x) + a + f64::from(u8::from(flag))
}

/// An explicit trailing `&Dots`: its `\usage` shows `...`, so the page
/// documents `...` and has no line for the Rust name `rest`.
/// @param x A number.
/// @param ... Values counted by Rust.
#[miniextendr(internal)]
pub fn dots_trailing_explicit(x: i32, rest: &Dots) -> i32 {
    x + dots_count(rest)
}

/// A `call = caller` wrapper with a formal after the dots: `.call` goes last,
/// after `flag`.
/// @param x Must be non-negative.
/// @param ... Values counted by Rust.
/// @param flag A flag.
#[miniextendr(noexport, call = caller)]
pub fn dots_mid_caller_impl(
    x: i32,
    rest: &Dots,
    #[miniextendr(default = "FALSE")] flag: bool,
) -> Result<String, String> {
    if x < 0 {
        return Err(format!("x must be non-negative, got {x}"));
    }
    Ok(format!("x={x} dots={} flag={flag}", dots_count(rest)))
}

/// `worker` on a function with dots: the dots keep it on the R main thread.
/// @param x A number.
/// @param ... Values counted by Rust.
/// @param flag A flag.
#[cfg(feature = "worker-thread")]
#[miniextendr(worker, noexport)]
pub fn dots_mid_worker(
    x: i32,
    rest: &Dots,
    #[miniextendr(default = "FALSE")] flag: bool,
) -> String {
    format!("x={x} dots={} flag={flag}", dots_count(rest))
}

// endregion

// region: methods, one class per class system

/// Env class with a formal after the method's dots.
#[derive(miniextendr_api::ExternalPtr)]
pub struct DotsPosEnv {
    base: i32,
}

#[miniextendr(env, noexport)]
impl DotsPosEnv {
    /// Create the fixture.
    /// @param base A number.
    pub fn new(base: i32) -> Self {
        Self { base }
    }

    /// Report `n`, the number of dots and `flag`.
    /// @param n A number.
    /// @param ... Values counted by Rust.
    /// @param flag A flag.
    #[miniextendr(defaults(flag = "FALSE"))]
    pub fn collect(&self, n: i32, rest: &Dots, flag: bool) -> String {
        format!(
            "base={} n={n} dots={} flag={flag}",
            self.base,
            dots_count(rest)
        )
    }
}

/// R6 class with a formal after the method's dots.
#[derive(miniextendr_api::ExternalPtr)]
pub struct DotsPosR6 {
    base: i32,
}

#[miniextendr(r6, noexport)]
impl DotsPosR6 {
    /// Create the fixture.
    /// @param base A number.
    pub fn new(base: i32) -> Self {
        Self { base }
    }

    /// Report `n`, the number of dots and `flag`.
    /// @param n A number.
    /// @param ... Values counted by Rust.
    /// @param flag A flag.
    #[miniextendr(defaults(flag = "FALSE"))]
    pub fn collect(&self, n: i32, rest: &Dots, flag: bool) -> String {
        format!(
            "base={} n={n} dots={} flag={flag}",
            self.base,
            dots_count(rest)
        )
    }
}

/// S3 class with a formal after the method's dots.
#[derive(miniextendr_api::ExternalPtr)]
pub struct DotsPosS3 {
    base: i32,
}

/// S3 class whose method takes a formal after its dots. Exported: roxygen2
/// wants every S3 method exported or registered.
#[miniextendr(s3)]
impl DotsPosS3 {
    /// Create the fixture.
    /// @param base A number.
    pub fn new(base: i32) -> Self {
        Self { base }
    }

    /// Report `n`, the number of dots and `flag`.
    /// @param n A number.
    /// @param ... Values counted by Rust.
    /// @param flag A flag.
    #[miniextendr(defaults(flag = "FALSE"))]
    pub fn dots_pos_s3_collect(&self, n: i32, rest: &Dots, flag: bool) -> String {
        format!(
            "base={} n={n} dots={} flag={flag}",
            self.base,
            dots_count(rest)
        )
    }
}

/// S4 class with a formal after the method's dots.
#[derive(miniextendr_api::ExternalPtr)]
pub struct DotsPosS4 {
    base: i32,
}

#[miniextendr(s4, noexport)]
impl DotsPosS4 {
    /// Create the fixture.
    /// @param base A number.
    pub fn new(base: i32) -> Self {
        Self { base }
    }

    /// Report `n`, the number of dots and `flag`.
    /// @param n A number.
    /// @param ... Values counted by Rust.
    /// @param flag A flag.
    #[miniextendr(defaults(flag = "FALSE"))]
    pub fn dots_pos_collect(&self, n: i32, rest: &Dots, flag: bool) -> String {
        format!(
            "base={} n={n} dots={} flag={flag}",
            self.base,
            dots_count(rest)
        )
    }
}

/// S7 class with a formal after the method's dots.
#[derive(miniextendr_api::ExternalPtr)]
pub struct DotsPosS7 {
    base: i32,
}

#[miniextendr(s7, noexport)]
impl DotsPosS7 {
    /// Create the fixture.
    /// @param base A number.
    pub fn new(base: i32) -> Self {
        Self { base }
    }

    /// Report `n`, the number of dots and `flag`.
    /// @param n A number.
    /// @param ... Values counted by Rust.
    /// @param flag A flag.
    #[miniextendr(defaults(flag = "FALSE"))]
    pub fn dots_pos_s7_collect(&self, n: i32, rest: &Dots, flag: bool) -> String {
        format!(
            "base={} n={n} dots={} flag={flag}",
            self.base,
            dots_count(rest)
        )
    }
}

/// vctrs class: vctrs impls take no instance methods (MXL120), so the formal
/// after the dots sits on a static helper.
pub struct DotsPosVctrs;

#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "dpv"), noexport)]
impl DotsPosVctrs {
    /// Create the fixture.
    /// @param values Numeric payload.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(values: Vec<f64>) -> Vec<f64> {
        values
    }

    /// Report `n`, the number of dots and `flag`.
    /// @param n A number.
    /// @param ... Values counted by Rust.
    /// @param flag A flag.
    #[miniextendr(defaults(flag = "FALSE"))]
    pub fn collect(n: i32, rest: &Dots, flag: bool) -> String {
        format!("n={n} dots={} flag={flag}", dots_count(rest))
    }
}

// endregion
