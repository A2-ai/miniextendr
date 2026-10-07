//! Test: `Option<Quoted>`.
//!
//! The wrapper passes an argument unevaluated only when the marker is the
//! parameter's whole type or the type argument of `Missing<..>` (#1835);
//! anywhere else it would be converted from a forced value.

use miniextendr_api::{Quoted, miniextendr};

#[miniextendr]
pub fn optional_quoted(cond: Option<Quoted>) -> i32 {
    let _ = cond;
    0
}

fn main() {}
