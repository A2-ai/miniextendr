//! Test: a trait impl method-level `no_na(...)` naming no parameter of the
//! method is rejected, as on inherent methods (the check would otherwise be
//! dropped without a word).

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn scale(&self, x_factor: f64) -> f64;
}

#[miniextendr(env)]
impl MyTrait for Dummy {
    #[miniextendr(no_na(x_factr))]
    fn scale(&self, x_factor: f64) -> f64 {
        x_factor
    }
}

fn main() {}
