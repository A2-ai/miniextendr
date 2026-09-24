//! Test: an S7 method on an Ops operator (`+`) must name its operand `e2`.
//! S7 dispatches the Ops group on `(e1, e2)`; the receiver becomes `e1`, and
//! the method's formals have to start with exactly those names.

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
