//! Test: a `&RngGuard` parameter on a trait-impl method.
//!
//! The impl method's arguments cross the trait ABI as R values like the
//! declaration's, and the guard is none (#2002, #2196).

use miniextendr_api::{ExternalPtr, miniextendr};

#[miniextendr]
pub trait Draw {
    fn draw(&self) -> f64;
}

#[derive(ExternalPtr)]
pub struct Sampler;

#[miniextendr]
impl Draw for Sampler {
    #[miniextendr(rng)]
    fn draw(&self, rng: &miniextendr_api::RngGuard) -> f64 {
        rng.unif_rand()
    }
}

fn main() {}
