//! Test: r! rejects a trailing binary operator (expression ends mid-op).

use miniextendr_macros::r;

fn main() {
    let _ = r!(x *);
}
