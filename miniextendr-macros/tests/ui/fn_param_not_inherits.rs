//! Test: rejected spellings of the parameter-level `not_inherits` check
//! (#1815): an unknown key, a message without a class, an empty class name,
//! and a class that `inherits` on the same parameter requires.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn unknown_key(#[miniextendr(not_inherits(class = "Date", msg = "m"))] x: Vec<f64>) {}

#[miniextendr]
pub fn message_without_class(#[miniextendr(not_inherits(message = "no dates"))] x: Vec<f64>) {}

#[miniextendr]
pub fn empty_class(#[miniextendr(not_inherits = "")] x: Vec<f64>) {}

#[miniextendr]
pub fn required_and_refused(
    #[miniextendr(inherits("pkg_a", "pkg_b"))]
    #[miniextendr(not_inherits = "pkg_b")]
    x: Vec<f64>,
) {
}

fn main() {}
