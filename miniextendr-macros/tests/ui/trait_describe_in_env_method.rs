//! Test: `@describeIn` on an Env trait-impl method is rejected. The method's
//! wrapper is a `Type$Trait$method` assignment, which roxygen2 cannot list in
//! the destination's "Functions" section (#1590).

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn value(&self) -> i32;
}

#[miniextendr(env)]
impl MyTrait for Dummy {
    /// @describeIn dummy_family The value.
    fn value(&self) -> i32 {
        42
    }
}

fn main() {}
