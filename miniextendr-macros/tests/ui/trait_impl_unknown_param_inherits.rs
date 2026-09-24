//! Test: a trait impl method-level `inherits(...)` naming no parameter of the
//! method is rejected, as on inherent methods (the check would otherwise be
//! dropped without a word).

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn fit(&self, model: f64) -> f64;
}

#[miniextendr(env)]
impl MyTrait for Dummy {
    #[miniextendr(inherits(modl = "pkg_model"))]
    fn fit(&self, model: f64) -> f64 {
        model
    }
}

fn main() {}
