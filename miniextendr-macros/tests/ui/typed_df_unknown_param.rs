//! Test: typed_df referencing a parameter not in the function signature.

use miniextendr_macros::miniextendr;

#[miniextendr(typed_df(nope = typed_dataframe!(x: i32)))]
pub fn f(df: i32) -> i32 {
    0
}

fn main() {}
