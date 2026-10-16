//! Compile-pass test for #2124: an `unsafe fn` is accepted only where no
//! generated wrapper calls it (a skipped trait method, an ignored inherent
//! method, raw `extern "C-unwind"` registration). The rejected forms are the
//! `unsafe_wrapper_*` compile-fail tests.

#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, SEXP, miniextendr};

#[derive(ExternalPtr)]
pub struct Host;

// region: trait methods

#[miniextendr]
pub trait RustOnly {
    #[miniextendr(skip)]
    unsafe fn unchecked(&self, value: i32) -> i32 { value }
    fn safe(&self) -> i32 { 1 }
}
impl RustOnly for Host {}
#[miniextendr(env, noexport)]
impl RustOnly for Host {}

#[miniextendr]
pub trait GenericRustOnly<T> {
    #[miniextendr(skip)]
    unsafe fn unchecked(&self, value: T) -> T { value }
    fn safe(&self) -> T;
}
#[miniextendr(env, noexport)]
impl GenericRustOnly<i32> for Host {
    unsafe fn unchecked(&self, value: i32) -> i32 { value }
    fn safe(&self) -> i32 { 2 }
}

// A populated impl of a non-generic trait inherits the declaration's `skip`.
#[miniextendr]
pub trait PopulatedRustOnly {
    #[miniextendr(skip)]
    unsafe fn populated_unchecked(&self) -> i32;
    fn populated_safe(&self) -> i32;
}
#[miniextendr(env, noexport)]
impl PopulatedRustOnly for Host {
    unsafe fn populated_unchecked(&self) -> i32 { 3 }
    fn populated_safe(&self) -> i32 { 4 }
}

// Implementing an `unsafe trait` is the author's promise; calling its
// methods is safe, so they get wrappers.
#[miniextendr]
pub unsafe trait Trusted {
    fn trusted(&self) -> i32;
}
#[miniextendr(env, noexport)]
unsafe impl Trusted for Host {
    fn trusted(&self) -> i32 { 5 }
}

// endregion

// region: inherent methods

#[derive(ExternalPtr)]
pub struct EnvHost;
#[miniextendr(env, noexport)]
impl EnvHost {
    #[miniextendr(env(ignore))]
    pub unsafe fn unchecked(&self) -> i32 { 1 }
    // A safe ABI-qualified method is an ordinary safe call.
    pub extern "C" fn c_abi(&self) -> i32 { 2 }
}

#[derive(ExternalPtr)]
pub struct R6Host;
#[miniextendr(r6, noexport)]
impl R6Host {
    #[miniextendr(r6(ignore))]
    pub unsafe fn unchecked(&self) -> i32 { 1 }
    pub fn safe(&self) -> i32 { 2 }
}

#[derive(ExternalPtr)]
pub struct S3Host;
#[miniextendr(s3, noexport)]
impl S3Host {
    #[miniextendr(s3(ignore))]
    pub unsafe fn unchecked(&self) -> i32 { 1 }
    pub fn s3_safe(&self) -> i32 { 2 }
}

#[derive(ExternalPtr)]
pub struct S4Host;
#[miniextendr(s4, noexport)]
impl S4Host {
    #[miniextendr(s4(ignore))]
    pub unsafe fn unchecked(&self) -> i32 { 1 }
    pub fn s4_host_safe(&self) -> i32 { 2 }
}

#[derive(ExternalPtr)]
pub struct S7Host;
#[miniextendr(s7, noexport)]
impl S7Host {
    #[miniextendr(s7(ignore))]
    pub unsafe fn unchecked(&self) -> i32 { 1 }
    pub fn s7_safe(&self) -> i32 { 2 }
}

// endregion

// region: standalone functions

#[miniextendr]
pub fn safe_adapter(value: i32) -> i32 {
    // This fixture's unchecked operation has no extra preconditions.
    unsafe { RustOnly::unchecked(&Host, value) }
}

// Raw registration generates no Rust wrapper around this entry point.
#[miniextendr]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn raw_identity(value: SEXP) -> SEXP {
    value
}

// endregion

fn main() {
    assert_eq!(safe_adapter(4), 4);
}
