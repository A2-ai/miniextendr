//! Compile-pass test for the unforced dots `LazyDots` (#1892): on a
//! standalone function (first, between the others, last, with `NArgs` and
//! `Missing` formals), on an `s3(...)` function, and on an impl-block method.
//! The wrapper passes `environment()` at the position of `...`.

#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, LazyDots, Missing, NArgs, SEXP, miniextendr};

#[miniextendr]
pub fn lazy_first(rest: LazyDots, x: i32) -> i32 {
    i32::try_from(rest.len()).unwrap_or(i32::MAX) + x
}

#[miniextendr]
pub fn lazy_names(x: i32, rest: LazyDots<'_>, flag: bool) -> Vec<String> {
    let _ = (x, flag);
    rest.names()
        .into_iter()
        .map(|name| name.unwrap_or("").to_owned())
        .collect()
}

#[miniextendr(s3(generic = "[", class = "lazy_grid"))]
pub fn lazy_grid_subset(
    x: SEXP,
    i: Missing<SEXP>,
    j: Missing<SEXP>,
    drop: Missing<SEXP>,
    rest: miniextendr_api::LazyDots,
    nargs: NArgs,
) -> SEXP {
    let _ = (i, j, drop, nargs.get());
    for k in 0..rest.len() {
        if !rest.is_missing_arg(k) {
            let _ = rest.force(k);
        }
    }
    x
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

    fn extra(&self, n: i32, rest: LazyDots) -> i32 {
        n + i32::try_from(rest.len()).unwrap_or(i32::MAX)
    }
}

fn main() {}
