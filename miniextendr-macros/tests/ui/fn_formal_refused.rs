//! Test: a declared `formal = ".."` (#1842) must be a syntactic R name the
//! generated wrapper doesn't use itself, distinct from the other formals,
//! given once, as a string, and not on the dots or an `NArgs` parameter.
//! A signature's problems are reported together.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn not_syntactic(#[miniextendr(formal = "conf int")] conf_int: f64) -> f64 {
    conf_int
}

#[miniextendr]
pub fn batched(
    #[miniextendr(formal = "if")] cond: bool,
    #[miniextendr(formal = "...")] rest: i32,
    #[miniextendr(formal = ".call")] call: i32,
    #[miniextendr(formal = ".mx_self")] this: i32,
    #[miniextendr(formal = ".Generic")] generic: i32,
) -> i32 {
    let _ = cond;
    rest + call + this + generic
}

#[miniextendr]
pub fn collides(x: f64, #[miniextendr(formal = "x")] other: f64) -> f64 {
    x + other
}

#[miniextendr]
pub fn twice(#[miniextendr(formal = "na.rm", formal = "rm.na")] na_rm: bool) -> bool {
    na_rm
}

#[miniextendr]
pub fn not_a_string(#[miniextendr(formal = na.rm)] na_rm: bool) -> bool {
    na_rm
}

#[miniextendr]
pub fn on_dots(x: f64, #[miniextendr(formal = "rest")] rest: &miniextendr_api::dots::Dots) -> f64 {
    let _ = rest;
    x
}

#[miniextendr]
pub fn on_nargs(x: f64, #[miniextendr(formal = "n.args")] n: miniextendr_api::NArgs) -> f64 {
    let _ = n;
    x
}

fn main() {}
