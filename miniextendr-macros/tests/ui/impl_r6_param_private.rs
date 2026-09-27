//! Test: an R6 method parameter named `private` is rejected.
//!
//! R6 binds `private` inside every method of the class, and the generated
//! method reads the object's pointer from `private$.ptr`; a parameter named
//! `private` (here `_private` without its underscore) would hide it.

use miniextendr_macros::miniextendr;

pub struct Counter;

#[miniextendr(r6)]
impl Counter {
    pub fn new() -> Self {
        Counter
    }

    pub fn bump(&self, _private: i32) -> i32 {
        0
    }
}

fn main() {}
