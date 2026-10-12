//! #1427: the trait-ABI shims bind a borrowed view to a root local to the
//! shim, so a trait method taking a `'static` view does not compile, in the
//! trait's own shim or in an impl's vtable shim.

use miniextendr_api::{ExternalPtr, StrVec, miniextendr};

#[miniextendr]
pub trait KeepStrings {
    fn keep(&self, strings: StrVec<'static>);
}

#[derive(ExternalPtr)]
pub struct Keeper;

#[miniextendr(env)]
impl Keeper {
    pub fn new() -> Self {
        Keeper
    }
}

#[miniextendr(env)]
impl KeepStrings for Keeper {
    fn keep(&self, strings: StrVec<'static>) {
        let _ = strings;
    }
}

fn main() {}
