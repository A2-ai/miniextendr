//! Test: a method-level option naming an `NArgs` parameter.
//!
//! `defaults(n = ...)` would write a formal the wrapper never makes: the
//! parameter is filled with `nargs()` (#1860).

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Bag {
    values: Vec<f64>,
}

#[miniextendr(s3)]
impl Bag {
    fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    #[miniextendr(s3(generic = "[["), defaults(n = "2L"))]
    fn at(&self, i: i32, n: miniextendr_api::NArgs) -> f64 {
        let _ = n;
        self.values[usize::try_from(i).unwrap_or(0)]
    }
}

fn main() {}
