//! Test: `no_default` drops the generated default of a `match_arg` /
//! `choices` parameter (#1828); on any other parameter it has nothing to drop
//! and must error at compile time, naming the parameter.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_no_default(#[miniextendr(no_default)] n: i32) {}

fn main() {}
