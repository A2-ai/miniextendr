//! Compile-pass test for declared R formals (#1842): `formal = ".."` with the
//! other per-parameter options on standalone functions (an S3 method among
//! them), under `Missing<..>`, on an unevaluated argument, and the method-level
//! `formal(p = "..")` on impl methods.

#![allow(dead_code)]

use miniextendr_api::{Missing, Quoted, SEXP, miniextendr};

#[miniextendr]
pub fn mean_of(
    x: Vec<f64>,
    #[miniextendr(formal = "na.rm", default = "FALSE")] na_rm: bool,
    #[miniextendr(formal = "trim.frac", no_na, coerce)] trim_frac: f64,
) -> f64 {
    let _ = (na_rm, trim_frac);
    x.iter().sum()
}

#[miniextendr(s3(generic = "summary", class = "formal_fit"))]
pub fn summary_formal_fit(
    object: SEXP,
    #[miniextendr(formal = "conf.int", default = "FALSE")] conf_int: bool,
    #[miniextendr(formal = "conf.level", default = "0.95")] conf_level: f64,
    _dots: ...
) -> f64 {
    let _ = (object, conf_int);
    conf_level
}

#[miniextendr]
pub fn optional_row_names(#[miniextendr(formal = "row.names")] row_names: Missing<SEXP>) -> bool {
    row_names.is_missing()
}

#[miniextendr]
pub fn quoted_data(#[miniextendr(formal = ".data")] data: Quoted) -> SEXP {
    data.expr()
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct Tally {
    total: f64,
}

#[miniextendr(r6)]
impl Tally {
    #[miniextendr(formal(start = "start.value"), defaults(start = "0"))]
    pub fn new(start: f64) -> Self {
        Tally { total: start }
    }

    #[miniextendr(formal(by = "by.value", na_rm = "na.rm"), defaults(na_rm = "FALSE"))]
    pub fn add(&mut self, by: f64, na_rm: bool) -> f64 {
        if !(na_rm && by.is_nan()) {
            self.total += by;
        }
        self.total
    }
}

fn main() {}
