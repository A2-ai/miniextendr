//! Visibility markers cannot be nested (#1213).
use miniextendr_api::miniextendr;

#[miniextendr]
pub fn nested() -> miniextendr_api::Invisible<miniextendr_api::Visible<i32>> {
    miniextendr_api::Invisible(miniextendr_api::Visible(1))
}

fn main() {}
