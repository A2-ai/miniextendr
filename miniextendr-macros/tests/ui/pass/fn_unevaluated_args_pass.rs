//! Compile-pass test for unevaluated arguments (#1835): `Quoted` and
//! `Quosure`, bare and under `Missing<..>`, with and without a written
//! lifetime, a fully qualified path, next to ordinary and `Call` parameters,
//! on a standalone S3 method, and under `worker` (which they keep on R's main
//! thread).

#![allow(dead_code)]

use miniextendr_api::{Call, Missing, Quosure, Quoted, SEXP, miniextendr};

#[miniextendr]
pub fn evaluate(expr: Quoted) -> SEXP {
    expr.eval()
}

#[miniextendr]
pub fn keep_rows(data: SEXP, cond: Quoted, limit: i32) -> i32 {
    let _ = cond.eval_in(data);
    limit
}

#[miniextendr]
pub fn column_name<'a>(col: Quoted<'a>) -> Option<String> {
    col.as_name().map(str::to_owned)
}

#[miniextendr]
pub fn optional_cond(cond: Missing<Quoted>) -> bool {
    cond.is_missing()
}

#[miniextendr]
pub fn qualified(expr: miniextendr_api::Quoted) -> SEXP {
    expr.expr()
}

#[miniextendr]
pub fn tidy_cols(data: SEXP, cols: Quosure, call: Call) -> SEXP {
    let _ = call.sexp();
    cols.eval_tidy(data)
}

#[miniextendr]
pub fn optional_cols(cols: Missing<Quosure<'_>>) -> bool {
    cols.is_present()
}

/// @param x A table.
/// @param subset A condition.
/// @param ... Ignored.
#[miniextendr(s3(generic = "subset", class = "mx_table"))]
pub fn mx_table_subset(x: SEXP, subset: Quoted, _dots: ...) -> SEXP {
    subset.eval_in(x)
}

#[miniextendr(worker)]
pub fn on_worker_request(expr: Quoted) -> i32 {
    i32::from(expr.as_name().is_some())
}

fn main() {}
