//! Test: unknown key in #[miniextendr(...)] on a trait method definition.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub trait MyTrait {
    #[miniextendr(typo_option)]
    fn value(&self) -> i32;
}

fn main() {}
