//! Test fixtures for ffi_guard module.

use miniextendr_api::ffi_guard::{GuardMode, guarded_ffi_call, guarded_ffi_call_with_fallback};
use miniextendr_api::panic_telemetry::PanicSource;
use miniextendr_api::prelude::*;

/// Test guarded_ffi_call with CatchUnwind on a non-panicking closure.
#[miniextendr]
pub fn ffi_guard_catch_unwind_ok() -> i32 {
    guarded_ffi_call(|| 42i32, GuardMode::CatchUnwind, PanicSource::Worker)
}

/// Test guarded_ffi_call_with_fallback on a non-panicking closure.
#[miniextendr]
pub fn ffi_guard_fallback_ok() -> i32 {
    guarded_ffi_call_with_fallback(|| 99i32, -1i32, PanicSource::Worker)
}

/// Test guarded_ffi_call_with_fallback on a panicking closure — returns fallback.
#[miniextendr]
pub fn ffi_guard_fallback_panic() -> i32 {
    guarded_ffi_call_with_fallback(
        || -> i32 { panic!("intentional panic") },
        -1i32,
        PanicSource::Worker,
    )
}
