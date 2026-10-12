//! Compile-pass regression test for #1118.
//!
//! An inherent `#[miniextendr]` impl block and a trait `#[miniextendr]` impl
//! block on the *same* type, each carrying an impl-*level* method-only roxygen
//! tag (`@param` / `@return`) placed before the `#[miniextendr]` attribute.
//! Each block strips its tag and emits a `#[deprecated]` nudge const; before
//! the fix both consts were named `_MINIEXTENDR_IMPL_METHOD_TAG_WARN_<Type>_0`
//! and collided with `error[E0428]: the name ... is defined multiple times`.
//! Each nudge is now wrapped in its own anonymous `const _: () = { .. };`
//! scope, so the inner const names can stay fixed — an anonymous const block
//! is a fresh item namespace every expansion — and this compiles without any
//! per-block disambiguator.
//!
//! Inherent + trait is the case exercised here; several inherent blocks of one
//! type are covered by `impl_blocks_merged_per_type.rs`.

#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, miniextendr};

#[miniextendr]
pub trait DualCounter {
    fn value(&self) -> i32;
}

#[derive(ExternalPtr)]
pub struct DualTagged {
    value: i32,
}

/// Constructor-side operations.
///
/// @param initial this belongs on a method, not the impl block
#[miniextendr(env)]
impl DualTagged {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
    fn get_value(&self) -> i32 {
        self.value
    }
}

/// Trait-side operations.
///
/// @return this belongs on a method, not the impl block
#[miniextendr(env)]
impl DualCounter for DualTagged {
    fn value(&self) -> i32 {
        self.value
    }
}

fn main() {}
