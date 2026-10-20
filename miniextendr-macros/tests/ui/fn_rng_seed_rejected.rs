//! Test: `rng(seed = ..)` on standalone functions it can't serve (#2198).
//!
//! Every problem of one function is reported at once: the seed names no
//! parameter, and a unit return can't carry the `seed` attribute. A seed must
//! also be an R argument the wrapper evaluates.

use miniextendr_api::miniextendr;

#[miniextendr(rng(seed = sed))]
pub fn misnamed(seed: Option<i64>) {
    let _ = seed;
}

#[miniextendr(rng(seed = rng))]
pub fn guard_as_seed(rng: &miniextendr_api::RngGuard) -> f64 {
    rng.unif_rand()
}

fn main() {}
