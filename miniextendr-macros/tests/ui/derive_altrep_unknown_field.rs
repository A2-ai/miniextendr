//! Test: a field selector (`field`, `len`, `elt`) must name a field of the
//! struct; tuple indices count as names.

use miniextendr_macros::{Altrep, AltrepInteger};

#[derive(Altrep)]
#[altrep(field = "values")]
struct Dice {
    value: Vec<i32>,
}

#[derive(AltrepInteger)]
#[altrep(len = "1")]
struct Outcomes(Vec<i32>);

#[derive(AltrepInteger)]
#[altrep(field = "n")]
struct Counts {
    data: Vec<i32>,
    n: usize,
}

fn main() {}
