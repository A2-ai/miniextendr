//! Test: a bare `#[derive(Altrep)]` reads a slice field, so a struct that
//! implements the data traits itself needs `#[altrep(manual)]`. Without it the
//! derive's `AltrepSlice` impl and the hand-written `AltrepLen` conflict, and
//! the error names `AltrepSlice`.

use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};
use miniextendr_macros::Altrep;

#[derive(Altrep)]
struct Counts {
    data: Vec<i32>,
}

impl AltrepLen for Counts {
    fn altrep_len(&self) -> usize {
        self.data.len()
    }
}

impl AltIntegerData for Counts {
    fn elt(&self, i: usize) -> i32 {
        self.data[i]
    }
}

fn main() {}
