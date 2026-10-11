//! Test: an R6 instance method whose R name is `initialize` is rejected.
//!
//! Every generated R6 class defines `initialize` (the constructor), so a
//! member under that name would make R6 refuse the class when the package
//! loads. Here an `r_name` renames `reset` onto it.

use miniextendr_macros::miniextendr;

pub struct Counter;

#[miniextendr(r6)]
impl Counter {
    pub fn new() -> Self {
        Counter
    }

    #[miniextendr(r_name = "initialize")]
    pub fn reset(&self) {}
}

fn main() {}
