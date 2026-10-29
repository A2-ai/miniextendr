#![allow(dead_code)]

//! Trait ABI coverage.
//!
//! Exercises `#[miniextendr]` on trait definitions and trait impls.
//! Both paths have separate expansion logic (vtable generation).

use super::impl_matrix::CovEnv;
use crate::miniextendr;

#[miniextendr]
pub trait CovTrait {
    fn bump(&mut self);
    fn cov_trait_value(&self) -> i32;
}

#[miniextendr]
impl CovTrait for CovEnv {
    fn bump(&mut self) {
        self.v += 1;
    }

    fn cov_trait_value(&self) -> i32 {
        self.v
    }
}

// region: Over-aligned types for alignment regression tests

/// Type with 32-byte alignment -- exposes alignment padding bugs in data
/// pointer computation.
#[repr(align(32))]
#[derive(crate::ExternalPtr)]
pub struct Aligned32Counter {
    pub value: i32,
}

#[miniextendr]
impl CovTrait for Aligned32Counter {
    fn bump(&mut self) {
        self.value += 1;
    }

    fn cov_trait_value(&self) -> i32 {
        self.value
    }
}

/// Type with 64-byte alignment (cache-line aligned).
#[repr(align(64))]
#[derive(crate::ExternalPtr)]
pub struct Aligned64Counter {
    pub value: i32,
}

#[miniextendr]
impl CovTrait for Aligned64Counter {
    fn bump(&mut self) {
        self.value += 1;
    }

    fn cov_trait_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: Tests

#[cfg(test)]
mod tests;
// endregion
