//! Test: a trait impl method takes the method-level `no_na(p(message = ...))`
//! form with the inherent-impl rules: an empty message is rejected.

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn scale(&self, x: f64) -> f64;
}

#[miniextendr(env)]
impl MyTrait for Dummy {
    #[miniextendr(no_na(x(message = "")))]
    fn scale(&self, x: f64) -> f64 {
        x
    }
}

fn main() {}
