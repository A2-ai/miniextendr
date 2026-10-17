//! A public method's @noRd must not suppress its R6 class's help page (#1820).
use miniextendr_macros::miniextendr;

struct Counter;

#[miniextendr(r6)]
impl Counter {
    /// @noRd
    pub fn value(&self) -> i32 {
        1
    }
}

fn main() {}
