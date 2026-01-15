//! Test: async function should fail.

use miniextendr_macros::miniextendr;

#[miniextendr]
async fn bad_async() -> i32 {
    42
}

fn main() {}
