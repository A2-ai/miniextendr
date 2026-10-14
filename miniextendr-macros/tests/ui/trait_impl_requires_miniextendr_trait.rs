//! Test: `#[miniextendr]` on an impl of a trait without `#[miniextendr]` is
//! rejected at the trait's name. The impl expands through the macro the
//! trait's own `#[miniextendr]` exports beside it, which a plain trait lacks.

use miniextendr_macros::miniextendr;

struct Dummy;

trait Plain {
    fn value(&self) -> i32;
}

#[miniextendr(env)]
impl Plain for Dummy {
    fn value(&self) -> i32 {
        42
    }
}

fn main() {}
