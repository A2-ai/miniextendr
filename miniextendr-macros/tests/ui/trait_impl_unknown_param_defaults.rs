//! Test: a trait impl method-level `defaults(...)` naming no parameter of the
//! method is rejected, as on inherent methods (the default would otherwise be
//! dropped without a word).

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn scale(&self, x: f64) -> f64;
}

#[miniextendr(env)]
impl MyTrait for Dummy {
    #[miniextendr(defaults(y = "1"))]
    fn scale(&self, x: f64) -> f64 {
        x
    }
}

fn main() {}
