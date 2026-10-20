//! Test: `rng(seed = ..)` on an inherent impl method.
//!
//! The seed contract is written in the R wrapper of a standalone function or
//! an `s3(...)` method only (#2198).

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Sampler;

#[miniextendr(env)]
impl Sampler {
    pub fn new() -> Self {
        Sampler
    }

    #[miniextendr(rng(seed = seed))]
    pub fn draw(&self, seed: Option<i64>) -> Option<i64> {
        seed
    }
}

fn main() {}
