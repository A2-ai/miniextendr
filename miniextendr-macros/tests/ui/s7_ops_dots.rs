//! Test: an S7 method on an Ops operator cannot take R's `...`. S7 defines
//! the operator's generic as `function(e1, e2)`, without `...`, and registers
//! a method on it only when the method's formals are exactly `e1, e2`.

use miniextendr_macros::miniextendr;

struct Money {
    cents: i32,
}

#[miniextendr(s7)]
impl Money {
    #[miniextendr(s7(generic = "+"))]
    fn add(&self, e2: f64, _dots: &miniextendr_api::Dots) -> f64 {
        f64::from(self.cents) + e2
    }
}

fn main() {}
