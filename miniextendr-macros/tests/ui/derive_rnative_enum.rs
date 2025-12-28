//! Test: derive RNativeType on enum should fail.

use miniextendr_macros::RNativeType;

#[derive(RNativeType)]
enum Bad {
    A,
}

fn main() {}
