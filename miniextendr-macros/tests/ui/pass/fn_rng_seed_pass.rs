//! Compile-pass test for `rng(seed = <param>)` (#2198): the seed contract of
//! `stats::simulate()` on a standalone function and on an `s3(...)` method.
//! The seed parameter reaches the body like any other, whatever its type and
//! position, and the option implies `rng`.

#![allow(dead_code)]

use miniextendr_api::dots::Dots;
use miniextendr_api::rng::RngGuard;
use miniextendr_api::{SEXP, miniextendr};

#[miniextendr(rng(seed = seed))]
pub fn seeded(
    n: i32,
    #[miniextendr(default = "NULL")] seed: Option<i64>,
    rng: &RngGuard,
) -> Vec<f64> {
    let _ = seed;
    (0..n).map(|_| rng.norm_rand()).collect()
}

#[miniextendr(rng(seed = _seed), s3(generic = "stats::simulate", class = "seed_fit"))]
pub fn simulate_seed_fit(
    _object: SEXP,
    #[miniextendr(default = "1")] nsim: usize,
    #[miniextendr(default = "NULL")] _seed: Option<i64>,
    _dots: &Dots,
    rng: &RngGuard,
) -> Vec<f64> {
    (0..nsim).map(|_| rng.unif_rand()).collect()
}

// A guard is optional: the state is loaded either way.
#[miniextendr(rng(seed = seed))]
pub fn seed_only(seed: Option<f64>) -> Option<f64> {
    seed
}

fn main() {}
