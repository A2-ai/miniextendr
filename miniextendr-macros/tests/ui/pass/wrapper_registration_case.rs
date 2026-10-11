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

#[cfg(feature = "vctrs")]
#[derive(miniextendr_api::Vctrs)]
#[vctrs(class = "case_percent_upper", base = "double")]
pub struct Percent { #[vctrs(data)] data: Vec<f64> }
#[cfg(feature = "vctrs")]
#[derive(miniextendr_api::Vctrs)]
#[vctrs(class = "case_percent_lower", base = "double")]
#[allow(non_camel_case_types)]
pub struct percent { #[vctrs(data)] data: Vec<f64> }

fn main() {}
