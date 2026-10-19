//! Test: a `&RngGuard` parameter on a trait method.
//!
//! A trait method's arguments cross the trait ABI as R values, and the guard
//! is none (#2002).

use miniextendr_api::miniextendr;

#[miniextendr]
pub trait Draw {
    fn draw(&self, rng: &miniextendr_api::RngGuard) -> f64;
}

fn main() {}
