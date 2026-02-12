//! Test: derive ExternalPtr on generic struct should fail.

use miniextendr_macros::ExternalPtr;

#[derive(ExternalPtr)]
struct MyGeneric<T> {
    data: T,
}

fn main() {}
