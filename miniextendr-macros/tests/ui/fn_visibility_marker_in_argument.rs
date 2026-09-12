//! `Invisible<T>` / `Visible<T>` mark the return type only (#1213).
use miniextendr_api::miniextendr;

#[miniextendr]
pub fn takes_marker(x: miniextendr_api::Invisible<i32>) -> i32 {
    x.0
}

fn main() {}
