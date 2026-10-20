//! Test: `rng(seed = ..)` on a trait-impl method.
//!
//! The seed contract is written in the R wrapper of a standalone function or
//! an `s3(...)` method only (#2198).

use miniextendr_api::{ExternalPtr, miniextendr};

#[miniextendr]
pub trait Draw {
    fn draw(&self, seed: Option<i64>) -> f64;
}

#[derive(ExternalPtr)]
pub struct Sampler;

#[miniextendr]
impl Draw for Sampler {
    #[miniextendr(rng(seed = seed))]
    fn draw(&self, seed: Option<i64>) -> f64 {
        seed.map_or(0.0, |s| s as f64)
    }
}

fn main() {}
