//! Test: extern function must have explicit return type.

use miniextendr_macros::miniextendr;

#[miniextendr]
#[unsafe(no_mangle)]
extern "C-unwind" fn C_bad() {}

fn main() {}
