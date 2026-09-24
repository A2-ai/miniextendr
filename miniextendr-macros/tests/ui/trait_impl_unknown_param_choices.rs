//! Test: a trait impl method-level `choices(...)` naming no parameter of the
//! method is rejected, as on inherent methods (the choice list would otherwise
//! be dropped without a word).

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn run(&self, mode: String) -> String;
}

#[miniextendr(env)]
impl MyTrait for Dummy {
    #[miniextendr(choices(mod = "fast, slow"))]
    fn run(&self, mode: String) -> String {
        mode
    }
}

fn main() {}
