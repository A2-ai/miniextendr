//! Test: a `&T` parameter borrows an external value, so a `T` that isn't
//! `TypedExternal` fails to compile at the parameter's type (#2106).

use miniextendr_api::miniextendr;

pub struct Plain {
    value: i32,
}

#[miniextendr]
pub fn read_plain(x: &Plain) -> i32 {
    x.value
}

fn main() {}
