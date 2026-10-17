//! Class-level and static-method @noRd remain valid for R6 (#1820).
#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Documented;

#[miniextendr(r6)]
impl Documented {
    pub fn new() -> Self {
        Self
    }

    pub fn value(&self) -> i32 {
        1
    }

    /// @noRd
    pub fn helper() -> i32 {
        2
    }
}

#[derive(ExternalPtr)]
pub struct Undocumented;

/// @noRd
#[miniextendr(r6)]
impl Undocumented {
    pub fn new() -> Self {
        Self
    }

    pub fn value(&self) -> i32 {
        1
    }
}

fn main() {}
