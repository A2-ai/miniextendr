//! Test: a precondition marker in return position is rejected: it marks a
//! parameter only, as a visibility marker marks the return type only.

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn scale(n: i32) -> miniextendr_api::Checked<i32> {
    miniextendr_api::Checked(n)
}

fn main() {}
