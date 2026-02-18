//! Test: method error_in_r and unwrap_in_r are mutually exclusive.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr]
impl MyType {
    fn new() -> Self {
        MyType
    }

    #[miniextendr(error_in_r, unwrap_in_r)]
    fn bad(&self) -> Result<i32, String> {
        Ok(42)
    }
}

fn main() {}
