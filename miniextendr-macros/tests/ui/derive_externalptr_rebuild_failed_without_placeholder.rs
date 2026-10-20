//! Test: `restored(rebuild_failed = ...)` needs a `%s` for the cause.

use miniextendr_macros::ExternalPtr;

#[derive(ExternalPtr)]
#[externalptr(restored(rebuild_failed = "not rebuilt"), rebuild = MyType::from_saved)]
struct MyType {
    val: i32,
}

fn main() {}
