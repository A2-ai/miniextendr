//! Test: two S4 instance methods on the same generic are rejected.
//!
//! The default generic follows the R method name, so `r_name = "kind"` sends
//! `category` to `s4_kind` as well, and its `setMethod("s4_kind", "Counter")`
//! would replace the first one.

use miniextendr_macros::miniextendr;

pub struct Counter;

#[miniextendr(s4)]
impl Counter {
    pub fn new() -> Self {
        Counter
    }

    pub fn kind(&self) -> i32 {
        0
    }

    #[miniextendr(r_name = "kind")]
    pub fn category(&self) -> i32 {
        1
    }
}

fn main() {}
