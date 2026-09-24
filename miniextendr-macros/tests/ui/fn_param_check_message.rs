//! Test: rejected `message = "..."` forms on the parameter-level `inherits` /
//! `no_na` checks: an empty message, a message without a class, an unknown
//! key, and a second message for the same check.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn empty_message(#[miniextendr(inherits(class = "pkg_obj", message = ""))] x: Vec<f64>) {}

#[miniextendr]
pub fn message_without_class(#[miniextendr(inherits(message = "needs a pkg_obj"))] x: Vec<f64>) {}

#[miniextendr]
pub fn unknown_inherits_key(#[miniextendr(inherits(class = "pkg_obj", msg = "m"))] x: Vec<f64>) {}

#[miniextendr]
pub fn unknown_no_na_key(#[miniextendr(no_na(msg = "m"))] x: f64) {}

#[miniextendr]
pub fn two_messages(
    #[miniextendr(inherits("pkg_a", message = "one"))]
    #[miniextendr(inherits("pkg_b", message = "two"))]
    x: Vec<f64>,
) {
}

fn main() {}
