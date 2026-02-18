//! Test: IntoList can only be derived for structs.

use miniextendr_macros::IntoList;

#[derive(IntoList)]
enum Bad {
    A,
    B,
}

fn main() {}
