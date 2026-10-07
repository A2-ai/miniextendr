//! Test: `default` with another option on a `Missing<Quoted>` parameter.
//!
//! `default` alone only writes the formal and is accepted (#1835); a check
//! next to it would force the argument.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn quoted_default_and_check(
    #[miniextendr(default = "NULL", inherits = "data.frame")] cond: miniextendr_api::Missing<
        miniextendr_api::Quoted,
    >,
) -> i32 {
    let _ = cond;
    0
}

fn main() {}
