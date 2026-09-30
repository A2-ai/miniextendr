//! Fixtures for the per-parameter R-side type checks (#1566): the
//! `Checked<T>` / `Unchecked<T>` markers, the per-parameter
//! `preconditions` / `no_preconditions`, and the method-level bare and list
//! forms, on free functions, every class system and a trait impl.
//!
//! A kept check words a bad argument in R (`'n' must be integer`), a dropped
//! one leaves it to the Rust conversion (`'n' must be a single integer: got
//! character`); both raise the same argument-error condition.

use miniextendr_api::{Checked, ExternalPtr, Unchecked, miniextendr};

// region: free functions

/// `Checked` keeps `n`'s checks under the function's `no_preconditions`;
/// `tol` follows the function.
/// @param n Iteration count.
/// @param tol Tolerance.
#[miniextendr(noexport, no_preconditions)]
pub fn pm_checked(n: Checked<i32>, tol: f64) -> f64 {
    f64::from(*n) * tol
}

/// `Unchecked` drops `xs`'s check; `factor` keeps its own.
/// @param factor Multiplier.
/// @param xs Values.
#[miniextendr(noexport)]
pub fn pm_unchecked(factor: f64, xs: Unchecked<Vec<f64>>) -> Vec<f64> {
    xs.into_inner().into_iter().map(|x| x * factor).collect()
}

/// The per-parameter keyword spelling of `pm_checked`: the same R wrapper.
/// @param n Iteration count.
/// @param tol Tolerance.
#[miniextendr(noexport, no_preconditions)]
pub fn pm_keyword_checked(#[miniextendr(preconditions)] n: i32, tol: f64) -> f64 {
    f64::from(n) * tol
}

/// The per-parameter keyword spelling of `pm_unchecked`.
/// @param factor Multiplier.
/// @param xs Values.
#[miniextendr(noexport)]
pub fn pm_keyword_unchecked(
    factor: f64,
    #[miniextendr(no_preconditions)] xs: Vec<f64>,
) -> Vec<f64> {
    xs.into_iter().map(|x| x * factor).collect()
}

/// `Unchecked` drops the type check; the named `no_na` check stays.
/// @param xs Values without NA.
#[miniextendr(noexport)]
pub fn pm_unchecked_no_na(#[miniextendr(no_na)] xs: Unchecked<Vec<f64>>) -> f64 {
    xs.iter().sum()
}

/// `Checked<u16>` under `coerce`: the widened (coerced) guard is kept.
/// @param n A non-negative whole number.
#[miniextendr(noexport, no_preconditions)]
pub fn pm_checked_coerce(#[miniextendr(coerce)] n: Checked<u16>) -> i32 {
    i32::from(*n)
}

/// `Checked<Option<i32>>`: `NULL`, or the integer checks.
/// @param n An integer, or NULL.
#[miniextendr(noexport, no_preconditions)]
pub fn pm_checked_option(n: Checked<Option<i32>>) -> i32 {
    n.unwrap_or(-1)
}

/// `Checked<&str>` on the worker thread: the marker wraps the borrow made
/// inside the worker closure.
/// @param s A string.
#[cfg(any(feature = "worker-thread", feature = "worker-default"))]
#[miniextendr(noexport, worker, no_preconditions)]
pub fn pm_checked_str_worker(s: Checked<&str>) -> String {
    s.to_uppercase()
}

/// A `call = caller` entry point with an unchecked parameter: the Rust
/// conversion's error names the call the helper's `.call` resolves to.
/// @param n An integer.
#[miniextendr(noexport, call = caller)]
pub fn pm_caller_unchecked(n: Unchecked<i32>) -> i32 {
    *n + 1
}

// endregion

// region: one class per class system, under the impl's `no_preconditions`
//
// Each class has a method whose `preconditions(n)` keeps `n`'s checks (its
// `scale` follows the impl) and a method whose `Checked` parameter keeps its
// own.

/// Env class under `no_preconditions`.
#[derive(ExternalPtr)]
pub struct PmEnv {
    seed: i32,
}

#[miniextendr(env, noexport, no_preconditions)]
impl PmEnv {
    /// Create the fixture.
    /// @param seed A seed.
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }

    /// `n` checked, `scale` not.
    /// @param n A count.
    /// @param scale A scale.
    #[miniextendr(preconditions(n))]
    pub fn draw(&self, n: i32, scale: f64) -> f64 {
        f64::from(n + self.seed) * scale
    }

    /// `seed` checked by its marker.
    /// @param seed A seed.
    pub fn reseed(&mut self, seed: Checked<i32>) -> i32 {
        self.seed = *seed;
        self.seed
    }
}

/// R6 class under `no_preconditions`.
#[derive(ExternalPtr)]
pub struct PmR6 {
    seed: i32,
}

#[miniextendr(r6, noexport, no_preconditions)]
impl PmR6 {
    /// Create the fixture.
    /// @param seed A seed.
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }

    /// `n` checked, `scale` not.
    /// @param n A count.
    /// @param scale A scale.
    #[miniextendr(preconditions(n))]
    pub fn draw(&self, n: i32, scale: f64) -> f64 {
        f64::from(n + self.seed) * scale
    }

    /// `seed` checked by its marker.
    /// @param seed A seed.
    pub fn reseed(&mut self, seed: Checked<i32>) -> i32 {
        self.seed = *seed;
        self.seed
    }
}

/// S3 class under `no_preconditions`. Exported: roxygen2 wants every S3
/// method exported or registered.
#[derive(ExternalPtr)]
pub struct PmS3 {
    seed: i32,
}

/// S3 class whose methods keep one parameter's checks under the impl's
/// `no_preconditions`.
#[miniextendr(s3, no_preconditions)]
impl PmS3 {
    /// Create the fixture.
    /// @param seed A seed.
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }

    /// `n` checked, `scale` not.
    /// @param n A count.
    /// @param scale A scale.
    #[miniextendr(preconditions(n))]
    pub fn pm_s3_draw(&self, n: i32, scale: f64) -> f64 {
        f64::from(n + self.seed) * scale
    }

    /// `seed` checked by its marker.
    /// @param seed A seed.
    pub fn pm_s3_reseed(&mut self, seed: Checked<i32>) -> i32 {
        self.seed = *seed;
        self.seed
    }
}

/// S4 class under `no_preconditions`.
#[derive(ExternalPtr)]
pub struct PmS4 {
    seed: i32,
}

#[miniextendr(s4, noexport, no_preconditions)]
impl PmS4 {
    /// Create the fixture.
    /// @param seed A seed.
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }

    /// `n` checked, `scale` not.
    /// @param n A count.
    /// @param scale A scale.
    #[miniextendr(preconditions(n))]
    pub fn pm_s4_draw(&self, n: i32, scale: f64) -> f64 {
        f64::from(n + self.seed) * scale
    }

    /// `seed` checked by its marker.
    /// @param seed A seed.
    pub fn pm_s4_reseed(&mut self, seed: Checked<i32>) -> i32 {
        self.seed = *seed;
        self.seed
    }
}

/// S7 class under `no_preconditions`.
#[derive(ExternalPtr)]
pub struct PmS7 {
    seed: i32,
}

#[miniextendr(s7, noexport, no_preconditions)]
impl PmS7 {
    /// Create the fixture.
    /// @param seed A seed.
    pub fn new(seed: i32) -> Self {
        Self { seed }
    }

    /// `n` checked, `scale` not.
    /// @param n A count.
    /// @param scale A scale.
    #[miniextendr(preconditions(n))]
    pub fn pm_s7_draw(&self, n: i32, scale: f64) -> f64 {
        f64::from(n + self.seed) * scale
    }

    /// `seed` checked by its marker.
    /// @param seed A seed.
    pub fn pm_s7_reseed(&mut self, seed: Checked<i32>) -> i32 {
        self.seed = *seed;
        self.seed
    }
}

/// vctrs class under `no_preconditions`: vctrs impls take no instance
/// methods (MXL120), so the checks are on static helpers.
pub struct PmVctrs;

#[miniextendr(
    vctrs(kind = "vctr", base = "double", abbr = "pmv"),
    noexport,
    no_preconditions
)]
impl PmVctrs {
    /// Create the fixture.
    /// @param values Numeric payload.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(values: Vec<f64>) -> Vec<f64> {
        values
    }

    /// `n` checked, `scale` not.
    /// @param n A count.
    /// @param scale A scale.
    #[miniextendr(preconditions(n))]
    pub fn draw(n: i32, scale: f64) -> f64 {
        f64::from(n) * scale
    }

    /// `seed` checked by its marker.
    /// @param seed A seed.
    pub fn reseed(seed: Checked<i32>) -> i32 {
        *seed
    }
}

// endregion

// region: trait impl

/// Scales a value, for the trait-impl precondition fixture.
#[miniextendr]
pub trait PmScale {
    /// `k` times `d`.
    fn pm_scaled(&self, k: f64, d: f64) -> f64;
}

/// Trait impl whose method drops `k`'s checks and keeps `d`'s.
#[miniextendr(r6, noexport)]
impl PmScale for PmR6 {
    #[miniextendr(no_preconditions(k))]
    fn pm_scaled(&self, k: f64, d: f64) -> f64 {
        k * d + f64::from(self.seed)
    }
}

// endregion
