//! Test: a precondition marker inside `Option` is rejected with a hint to
//! put it outermost; the conversion has no marker to unwrap there.

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn scale(n: Option<miniextendr_api::Checked<i32>>) -> i32 {
    n.map_or(0, |n| *n)
}

fn main() {}
