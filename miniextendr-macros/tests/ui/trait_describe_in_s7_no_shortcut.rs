//! Test: `@describeIn` on an S7 trait-impl instance method without its
//! fast-path shortcut is rejected. The shortcut is the method's only block
//! with an R function roxygen2 can list; `S7::method<-` has none (#1590).

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn value(&self) -> i32;
}

#[miniextendr(s7)]
impl MyTrait for Dummy {
    /// @describeIn dummy_family The value.
    #[miniextendr(s7(no_shortcut))]
    fn value(&self) -> i32 {
        42
    }
}

fn main() {}
