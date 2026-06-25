//! Compile-pass test: `#[miniextendr]` fn with an explicit lifetime param should compile.
//!
//! Lifetimes are erased at codegen — the generated `#[no_mangle] extern "C-unwind"` wrapper
//! produces a single monomorphic symbol regardless of lifetime parameters, so they are FFI-safe.
//! Only type/const generic parameters are rejected (they require monomorphization → multiple
//! symbols → incompatible with `#[no_mangle]`).

#![allow(dead_code)]

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn foo<'a>(x: &'a [f64]) -> Vec<f64> {
    x.to_vec()
}

fn main() {}
