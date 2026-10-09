//! Compile-pass test for the argument-count marker `NArgs` (#1860): on a
//! standalone function (first, between the others, after `value` on a
//! replacement method, next to `&Dots`, last) and on an impl-block method.
//! The parameter is no R formal; the C wrapper converts the `nargs()` the R
//! wrapper passes.

#![allow(dead_code)]

use miniextendr_api::dots::Dots;
use miniextendr_api::{ExternalPtr, Missing, NArgs, SEXP, miniextendr};

#[miniextendr]
pub fn count_first(n: NArgs, x: i32) -> i32 {
    i32::try_from(n.get()).unwrap_or(i32::MAX) + x
}

#[miniextendr(s3(generic = "[", class = "counted"))]
pub fn counted_subset(
    x: SEXP,
    i: Missing<SEXP>,
    j: Missing<SEXP>,
    nargs: miniextendr_api::NArgs,
    _dots: &Dots,
    drop: Missing<SEXP>,
) -> SEXP {
    let _ = (i, j, nargs.get(), drop);
    x
}

#[miniextendr(s3(generic = "[<-", class = "counted"))]
pub fn counted_assign(x: SEXP, i: Missing<SEXP>, value: SEXP, _nargs: NArgs) -> SEXP {
    let _ = (i, value);
    x
}

#[miniextendr]
pub fn count_last(x: f64, n: NArgs) -> f64 {
    x * f64::from(u32::try_from(usize::from(n)).unwrap_or(u32::MAX))
}

#[derive(ExternalPtr)]
pub struct Bag {
    values: Vec<f64>,
}

#[miniextendr(s3)]
impl Bag {
    fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    #[miniextendr(s3(generic = "[["))]
    fn at(&self, i: i32, nargs: NArgs) -> Result<f64, String> {
        if nargs.get() != 2 {
            return Err("one subscript".into());
        }
        usize::try_from(i)
            .ok()
            .and_then(|k| self.values.get(k).copied())
            .ok_or_else(|| "out of range".into())
    }
}

fn main() {}
