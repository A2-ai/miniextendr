//! Test: r! rejects an empty (non-trailing) function call argument.

use miniextendr_macros::r;

fn main() {
    let _ = r!(f(, 1));
}
