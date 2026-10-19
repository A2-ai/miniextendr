//! Test: a `&RngGuard` parameter on a function without `rng`.
//!
//! Only a `#[miniextendr(rng)]` wrapper loads R's RNG state and has a guard
//! to pass (#2002).

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn draw(n: i32, rng: &miniextendr_api::RngGuard) -> Vec<f64> {
    (0..n).map(|_| rng.unif_rand()).collect()
}

fn main() {}
