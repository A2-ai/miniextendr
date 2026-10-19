//! Test: `dots = empty` on an R6 method without `...`. R6 and env methods
//! have the formals of the Rust signature only, so there is no `...` to
//! check (#2157); an S3, S4 or S7 method has the generic's dispatch `...`.

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Counter {
    n: i32,
}

#[miniextendr(r6)]
impl Counter {
    pub fn new() -> Self {
        Counter { n: 0 }
    }

    #[miniextendr(dots = empty)]
    pub fn get(&self) -> i32 {
        self.n
    }
}

fn main() {}
