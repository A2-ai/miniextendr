//! Test: error_in_r and unwrap_in_r are mutually exclusive.

use miniextendr_macros::miniextendr;

#[miniextendr(error_in_r, unwrap_in_r)]
fn bad_fn() -> Result<i32, String> {
    Ok(42)
}

fn main() {}
