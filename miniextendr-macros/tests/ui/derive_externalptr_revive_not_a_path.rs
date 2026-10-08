//! Test: `revive` in `#[externalptr(...)]` takes the path of the rebuild
//! hook, not a string.

use miniextendr_macros::ExternalPtr;

#[derive(ExternalPtr)]
#[externalptr(r6, revive = "Self::revive")]
struct MyType {
    val: i32,
}

fn main() {}
