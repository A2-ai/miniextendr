//! Test: non-RNativeType argument should fail to compile.

use miniextendr_macros::miniextendr;

struct NotRNative;

#[miniextendr]
fn takes_bad(_x: NotRNative) -> i32 {
    0
}

fn main() {}
