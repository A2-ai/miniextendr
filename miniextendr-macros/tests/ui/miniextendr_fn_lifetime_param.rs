//! Test: #[miniextendr] fn with an explicit lifetime param should fail with a clear message.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn foo<'a>(x: &'a [f64]) -> Vec<f64> {
    x.to_vec()
}

fn main() {}
