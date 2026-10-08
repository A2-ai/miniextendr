//! Test: a per-parameter option on an `NArgs` parameter.
//!
//! The parameter is no R argument: the generated wrapper fills it with
//! `nargs()`, so `default` (or any other option) has nothing to act on (#1860).

use miniextendr_api::miniextendr;

#[miniextendr]
pub fn counted(x: i32, #[miniextendr(default = "2L")] n: miniextendr_api::NArgs) -> i32 {
    let _ = n;
    x
}

fn main() {}
