#![allow(dead_code)]

//! Trait ABI coverage.
//!
//! Exercises `#[miniextendr]` on trait definitions and trait impls.
//! Both paths have separate expansion logic (vtable generation).

use super::impl_matrix::CovEnv;
use crate::{miniextendr, miniextendr_module};

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

miniextendr_module! {
    mod trait_abi_matrix;

    impl CovTrait for CovEnv;
}
