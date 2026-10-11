//! Registration metadata must not collapse case-distinct Rust names.
#![allow(dead_code)]
#![deny(nonstandard_style)]

use miniextendr_api::{ExternalPtr, miniextendr};

#[miniextendr]
pub fn read() -> i32 { 1 }
#[miniextendr]
#[allow(non_snake_case)]
pub fn Read() -> i32 { 2 }

#[derive(ExternalPtr)]
pub struct Widget;
#[derive(ExternalPtr)]
#[allow(non_camel_case_types)]
pub struct widget;

// Case-distinct types need separate wrapper and class-name entries.
#[miniextendr(env)]
impl Widget {
    pub fn value(&self) -> i32 { 1 }
}
#[miniextendr(env)]
impl widget {
    pub fn value(&self) -> i32 { 2 }
}

#[derive(ExternalPtr)]
pub struct Split;

// Case-distinct labels on one type need separate wrapper entries too.
#[miniextendr(env, label = "ops")]
impl Split {
    pub fn first(&self) -> i32 { 1 }
}
#[miniextendr(env, label = "Ops")]
impl Split {
    pub fn second(&self) -> i32 { 2 }
}

// rpkg's vctrs_derive_example.rs covers case-distinct Vctrs types: this
// crate doesn't have the `vctrs` feature.

fn main() {}
