//! Test: two precondition markers on one parameter are rejected; one
//! decides.

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn scale(n: miniextendr_api::Checked<miniextendr_api::Unchecked<i32>>) -> i32 {
    **n
}

fn main() {}
