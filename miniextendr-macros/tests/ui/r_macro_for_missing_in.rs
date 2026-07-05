//! Test: r! rejects `for (x)` that is missing `in`.

use miniextendr_macros::r;

fn main() {
    let _ = r!(for (x) {});
}
