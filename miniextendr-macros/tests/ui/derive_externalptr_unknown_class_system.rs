//! Test: unknown class system in ExternalPtr derive.

use miniextendr_macros::ExternalPtr;

#[derive(ExternalPtr)]
#[externalptr(flutter)]
struct MyType {
    val: i32,
}

fn main() {}
