//! A return-type marker and the `visible` / `invisible` attribute must agree (#1213).
use miniextendr_api::miniextendr;

#[miniextendr(visible)]
pub fn disagree() -> miniextendr_api::Invisible<i32> {
    miniextendr_api::Invisible(1)
}

fn main() {}
