//! Compile-fail test: a method returning `Option<Class>` (#2121). There is no
//! `IntoR for Option<Class>` (the orphan rule refuses one in the class's
//! crate), so the return follows the `Option` rule only in the spellings the
//! note names: `Option<ExternalPtr<Class>>`, or an explicit class wrap.

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Board;

#[derive(ExternalPtr)]
pub struct Plan;

#[miniextendr(r6)]
impl Plan {
    pub fn new() -> Self {
        Plan
    }

    pub fn try_build(&self) -> Option<Board> {
        Some(Board)
    }
}

fn main() {}
