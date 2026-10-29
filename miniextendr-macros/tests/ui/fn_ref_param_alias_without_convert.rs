//! Test: the macro reads a `&T` parameter by its written type, so a `&T`
//! whose `T` is an alias of a type it reads from R (`str`, `[i32]`) is an
//! external value; the error names `#[miniextendr(convert)]` (#2110).

use miniextendr_api::miniextendr;

type Text = str;
type Values = [i32];

#[miniextendr]
pub fn text_len(x: &Text) -> usize {
    x.len()
}

#[miniextendr]
pub fn values_len(x: &Values) -> usize {
    x.len()
}

fn main() {}
