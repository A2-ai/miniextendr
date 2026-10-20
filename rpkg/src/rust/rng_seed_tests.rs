//! `rng(seed = <param>)` fixtures (#2198): the seed contract of
//! `stats::simulate()`, written in the generated R wrapper.
//!
//! - `simulate.mx_seed_fit`: a `simulate()` method drawing through the guard
//! - draws followed by an `Err` and by a panic: the session's `.Random.seed`
//!   is put back all the same
//! - a `NULL` result, which carries no `seed` attribute

use miniextendr_api::dots::Dots;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::rng::RngGuard;
use miniextendr_api::{Nullable, miniextendr};

/// Simulate standard normal draws for an `mx_seed_fit`, as
/// `stats::simulate()` does: with a `seed`, from `set.seed(seed)`, then the
/// session's random stream is put back.
///
/// @param object An object of class `mx_seed_fit`.
/// @param nsim Number of draws.
/// @param seed `NULL` to continue the session's random stream, or a seed for
///   `set.seed()`.
/// @param ... Unused.
/// @return `nsim` draws, with the `seed` attribute `stats::simulate()` gives.
#[miniextendr(rng(seed = _seed), s3(generic = "simulate", class = "mx_seed_fit"))]
pub fn simulate_mx_seed_fit(
    _object: SEXP,
    #[miniextendr(default = "1")] nsim: usize,
    #[miniextendr(default = "NULL")] _seed: Option<i64>,
    _dots: &Dots,
    rng: &RngGuard,
) -> Vec<f64> {
    (0..nsim).map(|_| rng.norm_rand()).collect()
}

/// Draw `n` uniforms under the seed contract, then return an error.
///
/// @param n Number of draws.
/// @param seed `NULL`, or a seed for `set.seed()`.
#[miniextendr(rng(seed = _seed))]
pub fn rng_seed_draw_then_err(
    n: i32,
    #[miniextendr(default = "NULL")] _seed: Option<i64>,
    rng: &RngGuard,
) -> Result<Vec<f64>, String> {
    let drawn: Vec<f64> = (0..n).map(|_| rng.unif_rand()).collect();
    Err(format!("failed after {} draws", drawn.len()))
}

/// Draw `n` uniforms under the seed contract, then panic.
///
/// @param n Number of draws.
/// @param seed `NULL`, or a seed for `set.seed()`.
#[miniextendr(rng(seed = _seed))]
pub fn rng_seed_draw_then_panic(
    n: i32,
    #[miniextendr(default = "NULL")] _seed: Option<i64>,
    rng: &RngGuard,
) -> Vec<f64> {
    let drawn: Vec<f64> = (0..n).map(|_| rng.unif_rand()).collect();
    panic!("panicked after {} draws", drawn.len())
}

/// Draw one uniform under the seed contract; `NULL` instead when `keep` is
/// `FALSE`, which carries no `seed` attribute.
///
/// @param keep Whether to return the draw.
/// @param seed `NULL`, or a seed for `set.seed()`.
#[miniextendr(rng(seed = _seed))]
pub fn rng_seed_draw_or_null(
    keep: bool,
    #[miniextendr(default = "NULL")] _seed: Option<i64>,
    rng: &RngGuard,
) -> Nullable<f64> {
    let draw = rng.unif_rand();
    Nullable(keep.then_some(draw))
}
