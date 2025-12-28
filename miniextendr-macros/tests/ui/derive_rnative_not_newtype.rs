//! Test: derive RNativeType on non-newtype struct should fail.

use miniextendr_macros::RNativeType;

#[derive(RNativeType)]
struct Bad {
    a: i32,
    b: i32,
}

fn main() {}
