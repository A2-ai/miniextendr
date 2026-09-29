//! Test: a precondition marker in a `#[miniextendr]` trait method signature
//! is rejected: the View passes arguments on as R values, and each impl's
//! wrapper checks them, so the impl spells it.

use miniextendr_api::miniextendr;

#[miniextendr]
pub trait Scale {
    fn scaled(&self, k: miniextendr_api::Checked<f64>) -> f64;
}

fn main() {}
