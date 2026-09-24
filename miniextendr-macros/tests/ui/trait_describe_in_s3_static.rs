//! Test: `@describeIn` on an S3 trait-impl static method is rejected. The
//! method's wrapper is a `Type$Trait$method` namespace member, which roxygen2
//! cannot list in the destination's "Functions" section (#1590).

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn make() -> i32;
}

#[miniextendr(s3)]
impl MyTrait for Dummy {
    /// @describeIn dummy_family Make one.
    fn make() -> i32 {
        42
    }
}

fn main() {}
