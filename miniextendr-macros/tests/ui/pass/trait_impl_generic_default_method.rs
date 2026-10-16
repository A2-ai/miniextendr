//! Generic annotated impls can omit defaults from the trait declaration.

use miniextendr_api::{ExternalPtr, miniextendr};

#[miniextendr]
pub trait Totals<T> {
    fn total(&self) -> T;
    fn label(&self) -> String {
        String::from("total")
    }
}

#[derive(ExternalPtr)]
pub struct Ledger(i32);

#[miniextendr(env)]
impl Totals<i32> for Ledger {
    fn total(&self) -> i32 {
        self.0
    }
}

#[miniextendr]
pub trait Fixed<const N: usize> {
    fn marker(&self) -> i32;
    fn values(&self, values: [i32; N]) -> [i32; N] { values }
}

#[miniextendr(env)]
impl Fixed<2> for Ledger {
    fn marker(&self) -> i32 { self.0 }
}

fn main() {}
