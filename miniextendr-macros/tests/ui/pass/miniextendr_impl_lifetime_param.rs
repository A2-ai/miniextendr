//! Compile-pass test: `#[miniextendr]` impl block with an explicit lifetime param should compile.
//!
//! Lifetimes are erased at codegen — the generated `#[no_mangle] extern "C-unwind"` C wrappers
//! produce monomorphic symbols regardless of lifetime parameters. The `type_ident` used in
//! generated code (bare struct name without generic args) is correct because lifetime arguments
//! are never needed in generated code (they are inferred or elided by the compiler).
//!
//! This test uses a static constructor method (no `&self` receiver) to avoid needing
//! `TypedExternal`, which is a constraint only on instance-method dispatch.

#![allow(dead_code)]

use miniextendr_macros::miniextendr;

struct Wrapper<'a> {
    data: &'a str,
}

#[miniextendr]
impl<'a> Wrapper<'a> {
    pub fn describe() -> i32 {
        42
    }
}

fn main() {}
