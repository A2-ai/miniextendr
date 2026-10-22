//! Test: `restored(rebuild_failed = ...)` words a failed rebuild, so it
//! needs the rebuild hook.

use miniextendr_macros::ExternalPtr;

#[derive(ExternalPtr)]
#[externalptr(restored(rebuild_failed = "not rebuilt: %s"))]
struct MyType {
    val: i32,
}

fn main() {}
