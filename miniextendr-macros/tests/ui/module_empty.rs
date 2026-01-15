//! Test: empty miniextendr_module! should fail (missing mod).

use miniextendr_macros::miniextendr_module;

miniextendr_module! {
    // Empty - missing mod declaration
}

fn main() {}
