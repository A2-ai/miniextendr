//! Test: an impl of a generic `#[miniextendr]` trait must spell out every
//! method the vtable holds, defaulted ones included. Its shims are generated
//! from the impl's own signatures, since the trait's type parameters are only
//! concrete there.

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

fn main() {}
