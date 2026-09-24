//! Test: an S7 instance method on an Ops operator (`+`) is a compile error.
//! S7 dispatches the Ops group on both operands, which needs a two-class
//! signature the generated single-class method cannot provide.

use miniextendr_macros::miniextendr;

struct Money {
    cents: i32,
}

#[miniextendr(s7)]
impl Money {
    #[miniextendr(r_name = "+")]
    fn add(&self, other: f64) -> f64 {
        f64::from(self.cents) + other
    }
}

fn main() {}
