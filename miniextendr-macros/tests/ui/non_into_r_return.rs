//! Test: non-IntoR return type should fail to compile.

use miniextendr_macros::miniextendr;

struct NotIntoR;

#[miniextendr]
fn returns_bad() -> NotIntoR {
    NotIntoR
}

fn main() {}
