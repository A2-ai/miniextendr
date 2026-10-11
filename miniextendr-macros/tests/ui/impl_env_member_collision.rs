//! Test: two Env members with the same R name are rejected.
//!
//! Constructors, instance methods and static methods are all assigned into the
//! class environment (`Counter$value <- function`), so the static method
//! renamed to `value` would replace the instance method of that name.

use miniextendr_macros::miniextendr;

pub struct Counter;

#[miniextendr(env)]
impl Counter {
    pub fn new() -> Self {
        Counter
    }

    pub fn value(&self) -> i32 {
        0
    }

    #[miniextendr(r_name = "value")]
    pub fn initial() -> i32 {
        0
    }
}

fn main() {}
