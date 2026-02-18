//! Test: #[export_name] not allowed on impl blocks.

use miniextendr_macros::miniextendr;

struct MyType;

#[miniextendr]
#[export_name = "custom"]
impl MyType {
    fn new() -> Self {
        MyType
    }
}

fn main() {}
