//! Test: `impl Trait` in argument position should fail.
//!
//! The C wrapper generates `let x: impl AsRef<str> = TryFromSexp::try_from_sexp(...)`
//! which is invalid — `impl Trait` cannot be used as a type annotation on `let` bindings.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn takes_impl_trait(x: impl AsRef<str>) -> String {
    x.as_ref().to_string()
}

fn main() {}
