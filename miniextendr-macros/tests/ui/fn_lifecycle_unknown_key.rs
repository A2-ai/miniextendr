//! Test: unknown key in lifecycle().

use miniextendr_macros::miniextendr;

#[miniextendr(lifecycle(unknown = "val"))]
fn my_fn() -> i32 {
    42
}

fn main() {}
