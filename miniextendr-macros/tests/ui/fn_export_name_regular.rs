//! Test: #[export_name] not allowed on regular #[miniextendr] functions.

use miniextendr_macros::miniextendr;

#[miniextendr]
#[export_name = "custom_name"]
fn my_fn() -> i32 {
    42
}

fn main() {}
