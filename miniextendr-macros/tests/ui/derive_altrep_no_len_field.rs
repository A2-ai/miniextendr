//! Test: an ALTREP family derive with no `len` / `length` field and no
//! `#[altrep(len = "...")]` reports at the struct name.

use miniextendr_macros::AltrepInteger;

/// No field gives the length.
#[derive(AltrepInteger)]
pub struct NoLen {
    pub size: usize,
}

fn main() {}
