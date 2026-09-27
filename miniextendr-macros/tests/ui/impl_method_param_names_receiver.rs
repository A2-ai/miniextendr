//! Test: an S3 instance method parameter named like the receiver is rejected.
//!
//! The generated S3 method is `m.Counter <- function(x, x, ...)`, which R
//! rejects as a repeated formal argument.

use miniextendr_macros::miniextendr;

pub struct Counter;

#[miniextendr(s3)]
impl Counter {
    pub fn new() -> Self {
        Counter
    }

    pub fn shift(&self, _x: i32) -> i32 {
        0
    }
}

fn main() {}
