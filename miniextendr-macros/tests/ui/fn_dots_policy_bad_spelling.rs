//! Test: the spellings `dots = ...` refuses (#2157). The problems of one
//! attribute are reported together.

use miniextendr_macros::miniextendr;

#[miniextendr(dots = forward)]
pub fn forward_without_names(_dots: ...) {}

#[miniextendr(dots = empty(allow = ["a", "a", 1, "..."], keep = "b"))]
pub fn bad_allow_list(_dots: ...) {}

#[miniextendr(dots = data(allow = "x"))]
pub fn data_with_options(_dots: ...) {}

#[miniextendr(dots = nothing)]
pub fn unknown_role(_dots: ...) {}

fn main() {}
