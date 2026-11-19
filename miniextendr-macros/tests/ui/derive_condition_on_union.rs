//! Test: RCondition supports structs and enums, not unions; the error
//! points at the `union` keyword.

use miniextendr_api::condition::RCondition;

#[derive(RCondition)]
union Bad {
    x: f64,
    y: i32,
}

fn main() {}
