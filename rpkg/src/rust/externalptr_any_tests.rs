//! Test fixtures for ExternalPtr Box<dyn Any> storage.
//!
//! Verifies the Any-based type checking, non-generic finalizer,
//! and downcast paths introduced by the Box<Box<dyn Any>> refactor.

use miniextendr_api::externalptr::{ErasedExternalPtr, ExternalPtr};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::prelude::*;

#[derive(ExternalPtr)]
pub struct TypeA {
    pub val: i32,
}

#[derive(ExternalPtr)]
pub struct TypeB {
    pub val: String,
}

#[miniextendr]
impl TypeA {
    pub fn new(val: i32) -> Self {
        TypeA { val }
    }
    pub fn get_val(&self) -> i32 {
        self.val
    }
}

#[miniextendr]
impl TypeB {
    pub fn new(val: String) -> Self {
        TypeB { val }
    }
    pub fn get_val(&self) -> String {
        self.val.clone()
    }
}

/// Test that into_inner recovers the value via Any::downcast.
#[miniextendr]
pub fn extptr_any_into_inner(val: i32) -> i32 {
    let ptr = ExternalPtr::new(TypeA { val });
    let inner = ExternalPtr::into_inner(ptr);
    inner.val
}

/// Test type-erased downcast via ErasedExternalPtr.
#[miniextendr]
pub fn extptr_any_erased_is(x: SEXP) -> bool {
    let erased = unsafe { ErasedExternalPtr::from_sexp(x) };
    erased.is::<TypeA>()
}

/// Test erased downcast_ref returns correct value.
#[miniextendr]
pub fn extptr_any_erased_downcast(x: SEXP) -> i32 {
    let erased = unsafe { ErasedExternalPtr::from_sexp(x) };
    erased.downcast_ref::<TypeA>().map(|a| a.val).unwrap_or(-1)
}

/// Test that wrong-type downcast returns None (not crash).
#[miniextendr]
pub fn extptr_any_wrong_type_is(x: SEXP) -> bool {
    let erased = unsafe { ErasedExternalPtr::from_sexp(x) };
    // TypeA ptr should NOT be TypeB
    !erased.is::<TypeB>()
}
