//! Compile-pass test for #1776: a type's methods split across several
//! `#[miniextendr]` impl blocks, with no per-block disambiguator.
//!
//! Each block registers its part of the class from an anonymous `const _`
//! scope, and its C wrappers are named after the type and the method, so the
//! blocks of one type don't collide (E0428) for any class system. Every kind
//! of member below appears in two blocks of one type, so an item emitted per
//! type rather than per method would be defined twice. The wrapper writer
//! merges the blocks into one class when the package installs; its checks
//! (one defining block, same class settings, no member bound twice) are
//! unit-tested in miniextendr-api's `registry.rs`.

#![allow(dead_code)]

use miniextendr_api::r_coerce::RCoerceError;
use miniextendr_api::{ExternalPtr, List, MatchArg, SEXP, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Mode {
    Fast,
    Safe,
}

// region: env

#[derive(ExternalPtr)]
pub struct SplitEnv {
    value: i32,
}

/// Constructor-side operations.
#[miniextendr(env)]
impl SplitEnv {
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    pub fn origin() -> i32 {
        0
    }
}

/// @param by this belongs on a method, not the impl block
#[miniextendr(env)]
impl SplitEnv {
    #[miniextendr(match_arg(mode))]
    pub fn bump(&self, mode: Mode) -> i32 {
        match mode {
            Mode::Fast => self.value + 1,
            Mode::Safe => self.value.saturating_add(1),
        }
    }

    #[miniextendr(lifecycle = "experimental")]
    pub fn doubled(&self) -> i32 {
        self.value * 2
    }

    #[miniextendr(as = "list")]
    pub fn as_list(&self) -> Result<List, RCoerceError> {
        Ok(List::from_values(vec![self.value]))
    }

    pub fn unit() -> i32 {
        1
    }
}

/// @return this belongs on a method, not the impl block
#[miniextendr(env)]
impl SplitEnv {
    #[miniextendr(match_arg(mode))]
    pub fn drop_by(&self, mode: Mode) -> i32 {
        match mode {
            Mode::Fast => self.value - 1,
            Mode::Safe => self.value.saturating_sub(1),
        }
    }

    #[miniextendr(lifecycle = "experimental")]
    pub fn tripled(&self) -> i32 {
        self.value * 3
    }

    #[miniextendr(as = "character")]
    pub fn as_character(&self) -> Result<SEXP, RCoerceError> {
        Err(RCoerceError::NotSupported {
            from: "SplitEnv",
            to: "character",
        })
    }

    pub fn ten() -> i32 {
        10
    }
}
// endregion

// region: R6

#[derive(ExternalPtr)]
pub struct SplitR6 {
    value: i32,
}

/// Constructor-side operations.
#[miniextendr(r6)]
impl SplitR6 {
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    #[miniextendr(r6(active))]
    pub fn current(&self) -> i32 {
        self.value
    }

    #[miniextendr(r6(private))]
    pub fn predecessor(&self) -> i32 {
        self.value - 1
    }

    #[miniextendr(lifecycle = "experimental")]
    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn origin() -> i32 {
        0
    }
}

#[miniextendr(r6)]
impl SplitR6 {
    #[miniextendr(r6(active))]
    pub fn doubled(&self) -> i32 {
        self.value * 2
    }

    #[miniextendr(r6(private))]
    pub fn successor(&self) -> i32 {
        self.value + 1
    }

    #[miniextendr(r6(finalize))]
    pub fn release(&mut self) {
        self.value = 0;
    }

    #[miniextendr(lifecycle = "experimental", match_arg(mode))]
    pub fn tripled(&self, mode: Mode) -> i32 {
        match mode {
            Mode::Fast => self.value * 3,
            Mode::Safe => self.value.saturating_mul(3),
        }
    }

    pub fn unit() -> i32 {
        1
    }
}
// endregion

// region: S3, S4, S7

#[derive(ExternalPtr)]
pub struct SplitS3 {
    value: i32,
}

/// Constructor-side operations.
#[miniextendr(s3)]
impl SplitS3 {
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    pub fn split3_value(&self) -> i32 {
        self.value
    }

    pub fn origin() -> i32 {
        0
    }
}

#[miniextendr(s3)]
impl SplitS3 {
    pub fn split3_doubled(&self) -> i32 {
        self.value * 2
    }

    pub fn unit() -> i32 {
        1
    }
}

#[derive(ExternalPtr)]
pub struct SplitS4 {
    value: i32,
}

/// Constructor-side operations.
#[miniextendr(s4)]
impl SplitS4 {
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    pub fn split4_value(&self) -> i32 {
        self.value
    }

    pub fn origin() -> i32 {
        0
    }
}

#[miniextendr(s4)]
impl SplitS4 {
    pub fn split4_doubled(&self) -> i32 {
        self.value * 2
    }

    pub fn unit() -> i32 {
        1
    }
}

#[derive(ExternalPtr)]
pub struct SplitS7 {
    value: i32,
}

/// Constructor-side operations.
#[miniextendr(s7)]
impl SplitS7 {
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    pub fn split7_value(&self) -> i32 {
        self.value
    }

    pub fn origin() -> i32 {
        0
    }
}

#[miniextendr(s7)]
impl SplitS7 {
    pub fn split7_doubled(&self) -> i32 {
        self.value * 2
    }

    pub fn unit() -> i32 {
        1
    }
}
// endregion

fn main() {}
