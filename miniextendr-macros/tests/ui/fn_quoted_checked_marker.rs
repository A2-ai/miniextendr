//! Test: `Checked<Quoted>`.
//!
//! `Checked` / `Unchecked` select a parameter's R-side checks; a `Quoted`
//! parameter has none, since the wrapper passes it unevaluated (#1835).

use miniextendr_api::{Checked, Quoted, miniextendr};

#[miniextendr]
pub fn checked_quoted(cond: Checked<Quoted>) -> i32 {
    let _ = cond;
    0
}

fn main() {}
