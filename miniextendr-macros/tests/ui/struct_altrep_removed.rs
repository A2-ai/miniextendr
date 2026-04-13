//! Test: #[miniextendr] rejects ALTREP class/base attributes with migration guidance.

use miniextendr_macros::miniextendr;

#[miniextendr(class = "MyAltrep")]
pub struct MyWrapper(Vec<i32>);

fn main() {}
