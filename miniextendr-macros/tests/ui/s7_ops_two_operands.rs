//! Test: an S7 method on an Ops operator takes exactly one operand. R calls
//! it as `e1 == e2`, so a second parameter could never be supplied.

use miniextendr_macros::miniextendr;

struct Money {
    cents: i32,
}

#[miniextendr(s7)]
impl Money {
    #[miniextendr(s7(generic = "=="))]
    fn equals(&self, e2: f64, tolerance: f64) -> bool {
        (f64::from(self.cents) - e2).abs() <= tolerance
    }
}

fn main() {}
