//! Test: a `&RngGuard` parameter on a class method without `rng`.
//!
//! Only a `#[miniextendr(rng)]` wrapper loads R's RNG state and has a guard
//! to pass (#2002).

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Sampler {
    scale: f64,
}

#[miniextendr(env)]
impl Sampler {
    fn new(scale: f64) -> Self {
        Self { scale }
    }

    fn draw(&self, rng: &miniextendr_api::RngGuard) -> f64 {
        self.scale * rng.unif_rand()
    }
}

fn main() {}
