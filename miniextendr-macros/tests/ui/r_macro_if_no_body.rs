//! Test: r! rejects `if (cond)` with no body.

use miniextendr_macros::r;

fn main() {
    let _ = r!(if (x));
}
