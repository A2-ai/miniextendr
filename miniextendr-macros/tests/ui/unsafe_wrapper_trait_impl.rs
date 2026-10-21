//! Test (#2124): an impl cannot add `unsafe` to a method its declaration
//! keeps safe, and the declaration rejects any other unsafe exposed method,
//! so the impl macro has no unsafety check of its own. rustc's E0053 points
//! at the safe declaration.

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Host;

#[miniextendr]
pub trait Safe {
    fn read(&self) -> i32;
}

#[miniextendr(env)]
impl Safe for Host {
    unsafe fn read(&self) -> i32 { 1 }
}

fn main() {}
