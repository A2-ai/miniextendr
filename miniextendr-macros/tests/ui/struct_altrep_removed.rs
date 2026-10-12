//! Test: #[miniextendr] rejects ALTREP class/base attributes with migration guidance.

use miniextendr_macros::miniextendr;

#[miniextendr(class = "MyAltrep")]
pub struct MyWrapper(Vec<i32>);

// Any field count: these options were never forwarded to the ExternalPtr path.
#[miniextendr(base = "Int")]
pub struct MyPair {
    a: i32,
    b: i32,
}

fn main() {}
