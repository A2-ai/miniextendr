//! Test: ALTREP derives refuse lifetime parameters: R keeps the struct in an
//! external pointer, which needs `'static` data.

use miniextendr_macros::{Altrep, AltrepReal};

#[derive(Altrep)]
struct View<'a>(&'a [i32]);

#[derive(AltrepReal)]
struct Reals<'a> {
    values: &'a [f64],
}

fn main() {}
