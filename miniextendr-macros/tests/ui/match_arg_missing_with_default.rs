//! Test: a `Missing<..>` match_arg parameter reports an omitted argument as
//! `Missing::Absent`, so a `default` would pick no choice and must error at
//! compile time (#2161). Any other `Missing<T>` parameter takes a default.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_omittable_default(
    #[miniextendr(match_arg, default = "\"a\"")] x: miniextendr_api::Missing<String>,
) {
}

fn main() {}
