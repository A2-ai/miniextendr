//! Test: a per-parameter option on a `Quoted` parameter.
//!
//! The wrapper passes the argument unevaluated; a default, a coercion or a
//! check would force it (#1835). `Missing<Quoted>` is the optional spelling.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn quoted_with_default(
    #[miniextendr(default = "NULL")] cond: miniextendr_api::Quoted,
) -> i32 {
    let _ = cond;
    0
}

fn main() {}
