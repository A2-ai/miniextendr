//! Test: an R6 static method named after one of the R6 generator object's own
//! bindings is rejected.
//!
//! Static methods are assigned onto the generator (`Counter$new <- function`),
//! so a non-constructor static `new` would replace R6's `$new()` and the class
//! could no longer create objects.

use miniextendr_macros::miniextendr;

pub struct Counter;

#[miniextendr(r6)]
impl Counter {
    #[miniextendr(r6(constructor))]
    pub fn create() -> Self {
        Counter
    }

    // Returns no `Self`, so it is a static method, not the constructor.
    pub fn new() -> i32 {
        0
    }
}

fn main() {}
