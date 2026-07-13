//! Compile-pass test: struct-level `#[miniextendr(prefer = "native")]`
//! (#1283 regression — used to E0119 unconditionally: the ExternalPtr
//! derive's IntoExternalPtr blanket IntoR collided with
//! derive_prefer_rnative's concrete IntoR).

#![allow(dead_code)]

use miniextendr_macros::miniextendr;

#[miniextendr(prefer = "native")]
#[derive(Copy, Clone, miniextendr_api::RNativeType)]
pub struct Wrapped(pub i32);

fn main() {}
